//! Intelligent field interpretation.
//!
//! The core intelligence that handles nuances in user-defined YAML,
//! detecting definition styles, inferring types, and classifying attributes.

use once_cell::sync::Lazy;
use regex::Regex;
use serde_yaml::Value as YamlValue;
use std::collections::HashSet;

// =============================================================================
// Pattern Regexes
// =============================================================================

static ARRAY_SUFFIX_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^([A-Z][a-zA-Z0-9]*)\[\]$").unwrap());
// Matches lowercase array types like "string[]", "decimal[]", "int[]"
static LOWERCASE_ARRAY_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^([a-z][a-zA-Z0-9]*)\[\]$").unwrap());
static ARROW_REF_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^->\s*([A-Z][a-zA-Z0-9]*)(\?)?$").unwrap());
static PIPE_ENUM_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^([a-zA-Z0-9_-]+(?:\s*\|\s*[a-zA-Z0-9_-]+)+)$").unwrap());
static INLINE_CONSTRAINT_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^([a-z]+)\s*[\[\(\{](.+)[\]\)\}]$").unwrap());
static TEMPLATE_VAR_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\$\{([^}]+)\}|\{\{([^}]+)\}\}").unwrap());
static JSON_LOGIC_VAR_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r#""var"\s*:\s*"([^"]+)""#).unwrap());

// Name pattern hints for type inference
// Organized by type with domain synonyms and common naming conventions
static NAME_PATTERNS: Lazy<Vec<(Regex, PrimitiveType)>> = Lazy::new(|| {
    vec![
        // =============================================================================
        // IDENTIFIER patterns - unique keys, references, codes
        // =============================================================================
        (Regex::new(r".*_id$").unwrap(), PrimitiveType::Identifier),
        (Regex::new(r".*_uuid$").unwrap(), PrimitiveType::Identifier),
        (Regex::new(r".*_key$").unwrap(), PrimitiveType::Identifier),
        (Regex::new(r".*_ref$").unwrap(), PrimitiveType::Identifier),
        (Regex::new(r".*_code$").unwrap(), PrimitiveType::Identifier),
        (Regex::new(r".*_slug$").unwrap(), PrimitiveType::Identifier),
        (Regex::new(r".*_handle$").unwrap(), PrimitiveType::Identifier),
        (Regex::new(r".*_token$").unwrap(), PrimitiveType::Identifier),
        (Regex::new(r".*_hash$").unwrap(), PrimitiveType::Identifier),
        (Regex::new(r".*_sku$").unwrap(), PrimitiveType::Identifier),
        (Regex::new(r".*_ean$").unwrap(), PrimitiveType::Identifier),
        (Regex::new(r".*_upc$").unwrap(), PrimitiveType::Identifier),
        (Regex::new(r".*_isbn$").unwrap(), PrimitiveType::Identifier),

        // =============================================================================
        // INTEGER patterns - counts, positions, quantities
        // =============================================================================
        (Regex::new(r".*_count$").unwrap(), PrimitiveType::Int),
        (Regex::new(r".*_index$").unwrap(), PrimitiveType::Int),
        (Regex::new(r".*_num$").unwrap(), PrimitiveType::Int),
        (Regex::new(r".*_number$").unwrap(), PrimitiveType::Int),
        (Regex::new(r".*_qty$").unwrap(), PrimitiveType::Int),
        (Regex::new(r".*_quantity$").unwrap(), PrimitiveType::Int),
        (Regex::new(r".*_position$").unwrap(), PrimitiveType::Int),
        (Regex::new(r".*_rank$").unwrap(), PrimitiveType::Int),
        (Regex::new(r".*_order$").unwrap(), PrimitiveType::Int),
        (Regex::new(r".*_priority$").unwrap(), PrimitiveType::Int),
        (Regex::new(r".*_level$").unwrap(), PrimitiveType::Int),
        (Regex::new(r".*_step$").unwrap(), PrimitiveType::Int),
        (Regex::new(r".*_sequence$").unwrap(), PrimitiveType::Int),
        (Regex::new(r".*_size$").unwrap(), PrimitiveType::Int),
        (Regex::new(r".*_length$").unwrap(), PrimitiveType::Int),
        (Regex::new(r".*_width$").unwrap(), PrimitiveType::Int),
        (Regex::new(r".*_height$").unwrap(), PrimitiveType::Int),
        (Regex::new(r".*_depth$").unwrap(), PrimitiveType::Int),
        (Regex::new(r".*_age$").unwrap(), PrimitiveType::Int),
        (Regex::new(r".*_year$").unwrap(), PrimitiveType::Int),
        (Regex::new(r".*_month$").unwrap(), PrimitiveType::Int),
        (Regex::new(r".*_day$").unwrap(), PrimitiveType::Int),
        (Regex::new(r".*_hour$").unwrap(), PrimitiveType::Int),
        (Regex::new(r".*_minute$").unwrap(), PrimitiveType::Int),
        (Regex::new(r".*_second$").unwrap(), PrimitiveType::Int),
        (Regex::new(r".*_millisecond$").unwrap(), PrimitiveType::Int),
        (Regex::new(r"^num_.*").unwrap(), PrimitiveType::Int),
        (Regex::new(r"^total_.*").unwrap(), PrimitiveType::Int),
        (Regex::new(r"^max_.*").unwrap(), PrimitiveType::Int),
        (Regex::new(r"^min_.*").unwrap(), PrimitiveType::Int),
        (Regex::new(r"^retry_.*").unwrap(), PrimitiveType::Int),
        (Regex::new(r".*_retries$").unwrap(), PrimitiveType::Int),
        (Regex::new(r".*_attempts$").unwrap(), PrimitiveType::Int),
        (Regex::new(r".*_limit$").unwrap(), PrimitiveType::Int),
        (Regex::new(r".*_cap$").unwrap(), PrimitiveType::Int),
        (Regex::new(r".*_duration$").unwrap(), PrimitiveType::Int), // duration in seconds/minutes
        (Regex::new(r".*_timeout$").unwrap(), PrimitiveType::Int),
        (Regex::new(r".*_interval$").unwrap(), PrimitiveType::Int),
        (Regex::new(r".*_ttl$").unwrap(), PrimitiveType::Int),
        (Regex::new(r".*_version$").unwrap(), PrimitiveType::Int),
        (Regex::new(r".*_revision$").unwrap(), PrimitiveType::Int),

        // =============================================================================
        // DATETIME patterns - timestamps, dates, times
        // =============================================================================
        (Regex::new(r".*_at$").unwrap(), PrimitiveType::Datetime),
        (Regex::new(r".*_on$").unwrap(), PrimitiveType::Datetime), // created_on, updated_on
        (Regex::new(r".*_date$").unwrap(), PrimitiveType::Datetime),
        (Regex::new(r".*_time$").unwrap(), PrimitiveType::Datetime),
        (Regex::new(r".*_datetime$").unwrap(), PrimitiveType::Datetime),
        (Regex::new(r".*_timestamp$").unwrap(), PrimitiveType::Datetime),
        (Regex::new(r"^timestamp$").unwrap(), PrimitiveType::Datetime),
        (Regex::new(r"^datetime$").unwrap(), PrimitiveType::Datetime),
        (Regex::new(r"created_.*").unwrap(), PrimitiveType::Datetime),
        (Regex::new(r"updated_.*").unwrap(), PrimitiveType::Datetime),
        (Regex::new(r"modified_.*").unwrap(), PrimitiveType::Datetime),
        (Regex::new(r"deleted_.*").unwrap(), PrimitiveType::Datetime),
        (Regex::new(r".*_updated$").unwrap(), PrimitiveType::Datetime),
        (Regex::new(r".*_modified$").unwrap(), PrimitiveType::Datetime),
        (Regex::new(r".*_created$").unwrap(), PrimitiveType::Datetime),
        (Regex::new(r".*_deleted$").unwrap(), PrimitiveType::Datetime),
        (Regex::new(r"expires_.*").unwrap(), PrimitiveType::Datetime),
        (Regex::new(r"expired_.*").unwrap(), PrimitiveType::Datetime),
        (Regex::new(r".*_expires$").unwrap(), PrimitiveType::Datetime),
        (Regex::new(r".*_expiry$").unwrap(), PrimitiveType::Datetime),
        (Regex::new(r".*_expiration$").unwrap(), PrimitiveType::Datetime),
        (Regex::new(r"starts_.*").unwrap(), PrimitiveType::Datetime),
        (Regex::new(r"ends_.*").unwrap(), PrimitiveType::Datetime),
        (Regex::new(r".*_started$").unwrap(), PrimitiveType::Datetime),
        (Regex::new(r".*_ended$").unwrap(), PrimitiveType::Datetime),
        (Regex::new(r".*_completed$").unwrap(), PrimitiveType::Datetime),
        (Regex::new(r".*_finished$").unwrap(), PrimitiveType::Datetime),
        (Regex::new(r"scheduled_.*").unwrap(), PrimitiveType::Datetime),
        (Regex::new(r".*_scheduled$").unwrap(), PrimitiveType::Datetime),
        (Regex::new(r"published_.*").unwrap(), PrimitiveType::Datetime),
        (Regex::new(r".*_published$").unwrap(), PrimitiveType::Datetime),
        (Regex::new(r".*_since$").unwrap(), PrimitiveType::Datetime),
        (Regex::new(r".*_until$").unwrap(), PrimitiveType::Datetime),
        // NOTE: Using specific patterns instead of ^last_.* and ^first_.* to avoid
        // conflicting with first_name, last_name which should be String.
        // These cover common datetime-related fields.
        (Regex::new(r"^last_login$").unwrap(), PrimitiveType::Datetime),
        (Regex::new(r"^last_seen$").unwrap(), PrimitiveType::Datetime),
        (Regex::new(r"^last_activity$").unwrap(), PrimitiveType::Datetime),
        (Regex::new(r"^last_access$").unwrap(), PrimitiveType::Datetime),
        (Regex::new(r"^last_used$").unwrap(), PrimitiveType::Datetime),
        (Regex::new(r"^last_visit$").unwrap(), PrimitiveType::Datetime),
        (Regex::new(r"^last_sync$").unwrap(), PrimitiveType::Datetime),
        (Regex::new(r"^last_backup$").unwrap(), PrimitiveType::Datetime),
        (Regex::new(r"^last_check$").unwrap(), PrimitiveType::Datetime),
        (Regex::new(r"^last_run$").unwrap(), PrimitiveType::Datetime),
        (Regex::new(r"^first_login$").unwrap(), PrimitiveType::Datetime),
        (Regex::new(r"^first_seen$").unwrap(), PrimitiveType::Datetime),
        (Regex::new(r"^first_visit$").unwrap(), PrimitiveType::Datetime),
        (Regex::new(r"^first_access$").unwrap(), PrimitiveType::Datetime),
        (Regex::new(r"^first_used$").unwrap(), PrimitiveType::Datetime),
        (Regex::new(r".*_when$").unwrap(), PrimitiveType::Datetime),
        (Regex::new(r"^birth_.*").unwrap(), PrimitiveType::Datetime),
        (Regex::new(r".*_birth$").unwrap(), PrimitiveType::Datetime),
        (Regex::new(r"^dob$").unwrap(), PrimitiveType::Datetime), // date of birth

        // =============================================================================
        // DECIMAL patterns - money, rates, scores, measurements
        // =============================================================================
        (Regex::new(r".*_rate$").unwrap(), PrimitiveType::Decimal),
        (Regex::new(r".*_ratio$").unwrap(), PrimitiveType::Decimal),
        (Regex::new(r".*_factor$").unwrap(), PrimitiveType::Decimal),
        (Regex::new(r".*_multiplier$").unwrap(), PrimitiveType::Decimal),
        (Regex::new(r".*_price$").unwrap(), PrimitiveType::Decimal),
        (Regex::new(r".*_cost$").unwrap(), PrimitiveType::Decimal),
        (Regex::new(r".*_fee$").unwrap(), PrimitiveType::Decimal),
        (Regex::new(r".*_tax$").unwrap(), PrimitiveType::Decimal),
        (Regex::new(r".*_discount$").unwrap(), PrimitiveType::Decimal),
        (Regex::new(r".*_amount$").unwrap(), PrimitiveType::Decimal),
        (Regex::new(r".*_balance$").unwrap(), PrimitiveType::Decimal),
        (Regex::new(r".*_total$").unwrap(), PrimitiveType::Decimal),
        (Regex::new(r".*_subtotal$").unwrap(), PrimitiveType::Decimal),
        (Regex::new(r".*_margin$").unwrap(), PrimitiveType::Decimal),
        (Regex::new(r".*_markup$").unwrap(), PrimitiveType::Decimal),
        (Regex::new(r".*_score$").unwrap(), PrimitiveType::Decimal),
        (Regex::new(r".*_rating$").unwrap(), PrimitiveType::Decimal),
        (Regex::new(r".*_weight$").unwrap(), PrimitiveType::Decimal),
        (Regex::new(r".*_percent$").unwrap(), PrimitiveType::Decimal),
        (Regex::new(r".*_percentage$").unwrap(), PrimitiveType::Decimal),
        (Regex::new(r".*_pct$").unwrap(), PrimitiveType::Decimal),
        (Regex::new(r".*_threshold$").unwrap(), PrimitiveType::Decimal),
        (Regex::new(r".*_latitude$").unwrap(), PrimitiveType::Decimal),
        (Regex::new(r".*_longitude$").unwrap(), PrimitiveType::Decimal),
        (Regex::new(r"^lat$").unwrap(), PrimitiveType::Decimal),
        (Regex::new(r"^lng$").unwrap(), PrimitiveType::Decimal),
        (Regex::new(r"^lon$").unwrap(), PrimitiveType::Decimal),
        (Regex::new(r".*_temperature$").unwrap(), PrimitiveType::Decimal),
        (Regex::new(r".*_temp$").unwrap(), PrimitiveType::Decimal),
        (Regex::new(r".*_avg$").unwrap(), PrimitiveType::Decimal),
        (Regex::new(r".*_average$").unwrap(), PrimitiveType::Decimal),
        (Regex::new(r".*_mean$").unwrap(), PrimitiveType::Decimal),
        (Regex::new(r".*_median$").unwrap(), PrimitiveType::Decimal),
        (Regex::new(r".*_variance$").unwrap(), PrimitiveType::Decimal),
        (Regex::new(r".*_std$").unwrap(), PrimitiveType::Decimal),
        (Regex::new(r".*_deviation$").unwrap(), PrimitiveType::Decimal),
        (Regex::new(r".*_confidence$").unwrap(), PrimitiveType::Decimal),
        (Regex::new(r".*_probability$").unwrap(), PrimitiveType::Decimal),
        (Regex::new(r".*_value$").unwrap(), PrimitiveType::Decimal), // monetary value

        // =============================================================================
        // BOOLEAN patterns - flags, states, capabilities
        // =============================================================================
        (Regex::new(r"^is_.*").unwrap(), PrimitiveType::Bool),
        (Regex::new(r"^has_.*").unwrap(), PrimitiveType::Bool),
        (Regex::new(r"^can_.*").unwrap(), PrimitiveType::Bool),
        (Regex::new(r"^should_.*").unwrap(), PrimitiveType::Bool),
        (Regex::new(r"^will_.*").unwrap(), PrimitiveType::Bool),
        (Regex::new(r"^was_.*").unwrap(), PrimitiveType::Bool),
        (Regex::new(r"^did_.*").unwrap(), PrimitiveType::Bool),
        (Regex::new(r"^allow_.*").unwrap(), PrimitiveType::Bool),
        (Regex::new(r"^enable_.*").unwrap(), PrimitiveType::Bool),
        (Regex::new(r"^disable_.*").unwrap(), PrimitiveType::Bool),
        (Regex::new(r"^show_.*").unwrap(), PrimitiveType::Bool),
        (Regex::new(r"^hide_.*").unwrap(), PrimitiveType::Bool),
        (Regex::new(r"^use_.*").unwrap(), PrimitiveType::Bool),
        (Regex::new(r"^include_.*").unwrap(), PrimitiveType::Bool),
        (Regex::new(r"^exclude_.*").unwrap(), PrimitiveType::Bool),
        (Regex::new(r"^require_.*").unwrap(), PrimitiveType::Bool),
        (Regex::new(r"^force_.*").unwrap(), PrimitiveType::Bool),
        (Regex::new(r"^skip_.*").unwrap(), PrimitiveType::Bool),
        (Regex::new(r"^needs_.*").unwrap(), PrimitiveType::Bool),
        (Regex::new(r".*_enabled$").unwrap(), PrimitiveType::Bool),
        (Regex::new(r".*_disabled$").unwrap(), PrimitiveType::Bool),
        (Regex::new(r".*_active$").unwrap(), PrimitiveType::Bool),
        (Regex::new(r".*_inactive$").unwrap(), PrimitiveType::Bool),
        (Regex::new(r".*_visible$").unwrap(), PrimitiveType::Bool),
        (Regex::new(r".*_hidden$").unwrap(), PrimitiveType::Bool),
        (Regex::new(r".*_public$").unwrap(), PrimitiveType::Bool),
        (Regex::new(r".*_private$").unwrap(), PrimitiveType::Bool),
        (Regex::new(r".*_required$").unwrap(), PrimitiveType::Bool),
        (Regex::new(r".*_optional$").unwrap(), PrimitiveType::Bool),
        (Regex::new(r".*_valid$").unwrap(), PrimitiveType::Bool),
        (Regex::new(r".*_invalid$").unwrap(), PrimitiveType::Bool),
        (Regex::new(r".*_verified$").unwrap(), PrimitiveType::Bool),
        (Regex::new(r".*_unverified$").unwrap(), PrimitiveType::Bool),
        (Regex::new(r".*_approved$").unwrap(), PrimitiveType::Bool),
        (Regex::new(r".*_rejected$").unwrap(), PrimitiveType::Bool),
        (Regex::new(r".*_confirmed$").unwrap(), PrimitiveType::Bool),
        (Regex::new(r".*_pending$").unwrap(), PrimitiveType::Bool),
        (Regex::new(r".*_locked$").unwrap(), PrimitiveType::Bool),
        (Regex::new(r".*_unlocked$").unwrap(), PrimitiveType::Bool),
        (Regex::new(r".*_blocked$").unwrap(), PrimitiveType::Bool),
        (Regex::new(r".*_suspended$").unwrap(), PrimitiveType::Bool),
        (Regex::new(r".*_archived$").unwrap(), PrimitiveType::Bool),
        (Regex::new(r".*_deprecated$").unwrap(), PrimitiveType::Bool),
        (Regex::new(r".*_flagged$").unwrap(), PrimitiveType::Bool),
        (Regex::new(r".*_featured$").unwrap(), PrimitiveType::Bool),
        (Regex::new(r".*_pinned$").unwrap(), PrimitiveType::Bool),
        (Regex::new(r".*_starred$").unwrap(), PrimitiveType::Bool),
        (Regex::new(r".*_bookmarked$").unwrap(), PrimitiveType::Bool),
        (Regex::new(r".*_favorite$").unwrap(), PrimitiveType::Bool),
        (Regex::new(r".*_default$").unwrap(), PrimitiveType::Bool),
        (Regex::new(r".*_primary$").unwrap(), PrimitiveType::Bool),
        (Regex::new(r".*_secondary$").unwrap(), PrimitiveType::Bool),
        (Regex::new(r".*_read$").unwrap(), PrimitiveType::Bool),
        (Regex::new(r".*_unread$").unwrap(), PrimitiveType::Bool),
        (Regex::new(r".*_seen$").unwrap(), PrimitiveType::Bool),
        (Regex::new(r".*_sent$").unwrap(), PrimitiveType::Bool),
        (Regex::new(r".*_delivered$").unwrap(), PrimitiveType::Bool),
        (Regex::new(r".*_failed$").unwrap(), PrimitiveType::Bool),
        (Regex::new(r".*_success$").unwrap(), PrimitiveType::Bool),
        (Regex::new(r".*_complete$").unwrap(), PrimitiveType::Bool),
        (Regex::new(r".*_incomplete$").unwrap(), PrimitiveType::Bool),
        (Regex::new(r".*_empty$").unwrap(), PrimitiveType::Bool),
        (Regex::new(r".*_available$").unwrap(), PrimitiveType::Bool),
        (Regex::new(r".*_unavailable$").unwrap(), PrimitiveType::Bool),
        (Regex::new(r".*_online$").unwrap(), PrimitiveType::Bool),
        (Regex::new(r".*_offline$").unwrap(), PrimitiveType::Bool),
        (Regex::new(r".*_connected$").unwrap(), PrimitiveType::Bool),
        (Regex::new(r".*_disconnected$").unwrap(), PrimitiveType::Bool),
        (Regex::new(r".*_synced$").unwrap(), PrimitiveType::Bool),
        (Regex::new(r".*_dirty$").unwrap(), PrimitiveType::Bool),
        (Regex::new(r".*_stale$").unwrap(), PrimitiveType::Bool),
        (Regex::new(r".*_fresh$").unwrap(), PrimitiveType::Bool),
        (Regex::new(r".*_cached$").unwrap(), PrimitiveType::Bool),
        (Regex::new(r".*_encrypted$").unwrap(), PrimitiveType::Bool),
        (Regex::new(r".*_signed$").unwrap(), PrimitiveType::Bool),
        (Regex::new(r".*_anonymous$").unwrap(), PrimitiveType::Bool),
        (Regex::new(r".*_authenticated$").unwrap(), PrimitiveType::Bool),
        (Regex::new(r".*_authorized$").unwrap(), PrimitiveType::Bool),
        (Regex::new(r".*_admin$").unwrap(), PrimitiveType::Bool),
        (Regex::new(r".*_guest$").unwrap(), PrimitiveType::Bool),
        (Regex::new(r".*_trial$").unwrap(), PrimitiveType::Bool),
        (Regex::new(r".*_premium$").unwrap(), PrimitiveType::Bool),
        (Regex::new(r".*_paid$").unwrap(), PrimitiveType::Bool),
        (Regex::new(r".*_free$").unwrap(), PrimitiveType::Bool),
        (Regex::new(r".*_test$").unwrap(), PrimitiveType::Bool),
        (Regex::new(r".*_sandbox$").unwrap(), PrimitiveType::Bool),
        (Regex::new(r".*_production$").unwrap(), PrimitiveType::Bool),
        (Regex::new(r".*_debug$").unwrap(), PrimitiveType::Bool),
        (Regex::new(r".*_flag$").unwrap(), PrimitiveType::Bool),

        // =============================================================================
        // STRING patterns - names, labels, descriptions, content
        // =============================================================================
        (Regex::new(r".*_name$").unwrap(), PrimitiveType::String),
        (Regex::new(r".*_title$").unwrap(), PrimitiveType::String),
        (Regex::new(r".*_label$").unwrap(), PrimitiveType::String),
        (Regex::new(r".*_description$").unwrap(), PrimitiveType::String),
        (Regex::new(r".*_desc$").unwrap(), PrimitiveType::String),
        (Regex::new(r".*_summary$").unwrap(), PrimitiveType::String),
        (Regex::new(r".*_text$").unwrap(), PrimitiveType::String),
        (Regex::new(r".*_content$").unwrap(), PrimitiveType::String),
        (Regex::new(r".*_body$").unwrap(), PrimitiveType::String),
        (Regex::new(r".*_message$").unwrap(), PrimitiveType::String),
        (Regex::new(r".*_note$").unwrap(), PrimitiveType::String),
        (Regex::new(r".*_notes$").unwrap(), PrimitiveType::String),
        (Regex::new(r".*_comment$").unwrap(), PrimitiveType::String),
        (Regex::new(r".*_comments$").unwrap(), PrimitiveType::String),
        (Regex::new(r".*_remark$").unwrap(), PrimitiveType::String),
        (Regex::new(r".*_remarks$").unwrap(), PrimitiveType::String),
        (Regex::new(r".*_email$").unwrap(), PrimitiveType::String),
        (Regex::new(r".*_phone$").unwrap(), PrimitiveType::String),
        (Regex::new(r".*_mobile$").unwrap(), PrimitiveType::String),
        (Regex::new(r".*_fax$").unwrap(), PrimitiveType::String),
        (Regex::new(r".*_url$").unwrap(), PrimitiveType::String),
        (Regex::new(r".*_uri$").unwrap(), PrimitiveType::String),
        (Regex::new(r".*_link$").unwrap(), PrimitiveType::String),
        (Regex::new(r".*_href$").unwrap(), PrimitiveType::String),
        (Regex::new(r".*_path$").unwrap(), PrimitiveType::String),
        (Regex::new(r".*_file$").unwrap(), PrimitiveType::String),
        (Regex::new(r".*_filename$").unwrap(), PrimitiveType::String),
        (Regex::new(r".*_directory$").unwrap(), PrimitiveType::String),
        (Regex::new(r".*_folder$").unwrap(), PrimitiveType::String),
        (Regex::new(r".*_address$").unwrap(), PrimitiveType::String),
        (Regex::new(r".*_street$").unwrap(), PrimitiveType::String),
        (Regex::new(r".*_city$").unwrap(), PrimitiveType::String),
        (Regex::new(r".*_province$").unwrap(), PrimitiveType::String),
        (Regex::new(r".*_country$").unwrap(), PrimitiveType::String),
        (Regex::new(r".*_zip$").unwrap(), PrimitiveType::String),
        (Regex::new(r".*_postal$").unwrap(), PrimitiveType::String),
        (Regex::new(r".*_region$").unwrap(), PrimitiveType::String),
        (Regex::new(r".*_locale$").unwrap(), PrimitiveType::String),
        (Regex::new(r".*_language$").unwrap(), PrimitiveType::String),
        (Regex::new(r".*_lang$").unwrap(), PrimitiveType::String),
        (Regex::new(r".*_timezone$").unwrap(), PrimitiveType::String),
        (Regex::new(r".*_tz$").unwrap(), PrimitiveType::String),
        (Regex::new(r".*_currency$").unwrap(), PrimitiveType::String),
        (Regex::new(r".*_unit$").unwrap(), PrimitiveType::String),
        (Regex::new(r".*_format$").unwrap(), PrimitiveType::String),
        (Regex::new(r".*_type$").unwrap(), PrimitiveType::String),
        (Regex::new(r".*_kind$").unwrap(), PrimitiveType::String),
        (Regex::new(r".*_category$").unwrap(), PrimitiveType::String),
        (Regex::new(r".*_class$").unwrap(), PrimitiveType::String),
        (Regex::new(r".*_group$").unwrap(), PrimitiveType::String),
        (Regex::new(r".*_status$").unwrap(), PrimitiveType::String),
        (Regex::new(r".*_state$").unwrap(), PrimitiveType::String),
        (Regex::new(r".*_mode$").unwrap(), PrimitiveType::String),
        (Regex::new(r".*_reason$").unwrap(), PrimitiveType::String),
        (Regex::new(r".*_source$").unwrap(), PrimitiveType::String),
        (Regex::new(r".*_target$").unwrap(), PrimitiveType::String),
        (Regex::new(r".*_origin$").unwrap(), PrimitiveType::String),
        (Regex::new(r".*_destination$").unwrap(), PrimitiveType::String),
        (Regex::new(r".*_prefix$").unwrap(), PrimitiveType::String),
        (Regex::new(r".*_suffix$").unwrap(), PrimitiveType::String),
        (Regex::new(r".*_pattern$").unwrap(), PrimitiveType::String),
        (Regex::new(r".*_regex$").unwrap(), PrimitiveType::String),
        (Regex::new(r".*_template$").unwrap(), PrimitiveType::String),
        (Regex::new(r".*_schema$").unwrap(), PrimitiveType::String),
        (Regex::new(r".*_icon$").unwrap(), PrimitiveType::String),
        (Regex::new(r".*_image$").unwrap(), PrimitiveType::String),
        (Regex::new(r".*_avatar$").unwrap(), PrimitiveType::String),
        (Regex::new(r".*_logo$").unwrap(), PrimitiveType::String),
        (Regex::new(r".*_color$").unwrap(), PrimitiveType::String),
        (Regex::new(r".*_colour$").unwrap(), PrimitiveType::String),
        (Regex::new(r".*_font$").unwrap(), PrimitiveType::String),
        (Regex::new(r".*_style$").unwrap(), PrimitiveType::String),
        (Regex::new(r".*_theme$").unwrap(), PrimitiveType::String),
        (Regex::new(r"^first_name$").unwrap(), PrimitiveType::String),
        (Regex::new(r"^last_name$").unwrap(), PrimitiveType::String),
        (Regex::new(r"^middle_name$").unwrap(), PrimitiveType::String),
        (Regex::new(r"^full_name$").unwrap(), PrimitiveType::String),
        (Regex::new(r"^display_name$").unwrap(), PrimitiveType::String),
        (Regex::new(r"^nickname$").unwrap(), PrimitiveType::String),
        (Regex::new(r"^username$").unwrap(), PrimitiveType::String),
        (Regex::new(r"^password$").unwrap(), PrimitiveType::String),
        (Regex::new(r"^bio$").unwrap(), PrimitiveType::String),
        (Regex::new(r"^about$").unwrap(), PrimitiveType::String),
        (Regex::new(r"^headline$").unwrap(), PrimitiveType::String),
        (Regex::new(r"^tagline$").unwrap(), PrimitiveType::String),
        (Regex::new(r"^motto$").unwrap(), PrimitiveType::String),
        (Regex::new(r"^slogan$").unwrap(), PrimitiveType::String),
    ]
});

// =============================================================================
// Core Types
// =============================================================================

/// Primitive types for field classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimitiveType {
    String,
    Int,
    Decimal,
    Bool,
    Datetime,
    Identifier,
    Object,
    Array,
}

impl PrimitiveType {
    /// Convert to datatype ID string.
    pub fn to_datatype_id(&self) -> &'static str {
        match self {
            Self::String => "string",
            Self::Int => "int",
            Self::Decimal => "decimal",
            Self::Bool => "boolean",
            Self::Datetime => "datetime",
            Self::Identifier => "identifier",
            Self::Object => "object",
            Self::Array => "array",
        }
    }
}

/// Detected definition style.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefinitionStyle {
    /// { type: "string", ... }
    ExplicitTyped,
    /// "string" or "enum[a,b,c]"
    InlineShorthand,
    /// "The user's age in years"
    NaturalLanguage,
    /// { description: ..., options: ... }
    NestedObject,
    /// "entity.attribute: type"
    FlatDotPath,
    /// "a | b | c"
    PipeSeparatedEnum,
    /// "Entity[]"
    ArraySuffix,
    /// "-> Entity"
    ArrowReference,
    /// Has formula/expression/computed key
    ComputedFormula,
    /// Has min/max/pattern inline
    ConstraintInline,
    /// Unknown/default
    Unknown,
}

/// Confidence level for inferences.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Confidence {
    /// Explicitly specified in YAML.
    Certain,
    /// Strong name/pattern match.
    High,
    /// Value-based or contextual.
    Medium,
    /// Description-based guess.
    Low,
    /// Default fallback.
    Default,
}

impl Confidence {
    /// Get numeric weight for aggregation.
    pub fn weight(&self) -> f32 {
        match self {
            Self::Certain => 1.0,
            Self::High => 0.8,
            Self::Medium => 0.5,
            Self::Low => 0.3,
            Self::Default => 0.1,
        }
    }
}

/// Inferred type with confidence.
#[derive(Debug, Clone)]
pub struct InferredType {
    pub primitive: PrimitiveType,
    pub confidence: Confidence,
    pub signals: Vec<String>,
}

/// Cardinality for relationships.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cardinality {
    OneToOne,
    OneToMany,
    ManyToOne,
    ManyToMany,
}

impl Cardinality {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OneToOne => "one-to-one",
            Self::OneToMany => "one-to-many",
            Self::ManyToOne => "many-to-one",
            Self::ManyToMany => "many-to-many",
        }
    }
}

/// Detected relationship.
#[derive(Debug, Clone)]
pub struct DetectedRelationship {
    pub target: String,
    pub cardinality: Cardinality,
    pub optional: bool,
    pub confidence: Confidence,
    pub signals: Vec<String>,
}

/// Attribute semantics (static vs instance).
#[derive(Debug, Clone)]
pub enum AttributeSemantics {
    /// Template-level, shared across instances.
    Static { confidence: f32 },
    /// Per-instance value.
    Instance { confidence: f32 },
}

impl AttributeSemantics {
    pub fn is_static(&self) -> bool {
        matches!(self, Self::Static { .. })
    }

    pub fn is_instance(&self) -> bool {
        matches!(self, Self::Instance { .. })
    }
}

/// Interpreted field result.
#[derive(Debug, Clone)]
pub struct InterpretedField {
    /// Original key name.
    pub key: String,
    /// Detected definition style.
    pub style: DefinitionStyle,
    /// Inferred type.
    pub inferred_type: InferredType,
    /// Static vs instance classification.
    pub semantics: AttributeSemantics,
    /// Detected relationship (if any).
    pub relationship: Option<DetectedRelationship>,
    /// Enum values (if enum type).
    pub enum_values: Option<Vec<String>>,
    /// Default value.
    pub default_value: Option<serde_json::Value>,
    /// Computed formula/expression.
    pub formula: Option<String>,
    /// Constraints (min, max, pattern).
    pub constraints: FieldConstraints,
    /// Description.
    pub description: Option<String>,
    /// Is required.
    pub required: bool,
    /// All interpretation signals for reporting.
    pub signals: Vec<InterpretationSignal>,
    /// Warnings about type conflicts or semantic issues.
    pub warnings: Vec<TypeConflictWarning>,
    /// Nested property interpretations (for object types with properties).
    pub nested_properties: Option<Vec<InterpretedField>>,
}

/// Constraints on a field.
#[derive(Debug, Clone, Default)]
pub struct FieldConstraints {
    pub min: Option<serde_json::Value>,
    pub max: Option<serde_json::Value>,
    pub pattern: Option<String>,
}

/// A signal used in interpretation (for reporting).
#[derive(Debug, Clone)]
pub struct InterpretationSignal {
    pub source: String,
    pub signal: String,
    pub confidence: Confidence,
}

/// A warning about potential type conflicts or semantic issues.
#[derive(Debug, Clone)]
pub struct TypeConflictWarning {
    /// The field name.
    pub field: String,
    /// The explicit type that was specified.
    pub explicit_type: String,
    /// The type that would have been inferred from name patterns.
    pub inferred_type: String,
    /// Description of the potential issue.
    pub message: String,
}

// =============================================================================
// Intelligent Interpreter
// =============================================================================

/// Context for interpretation.
#[derive(Debug, Clone, Default)]
pub struct InterpretationContext {
    /// Known entity names.
    pub known_entities: HashSet<String>,
    /// Known enum type names.
    pub known_enums: HashSet<String>,
    /// Current entity being processed.
    pub current_entity: Option<String>,
    /// Fields referenced in formulas.
    pub formula_references: HashSet<String>,
}

/// The intelligent interpreter.
#[derive(Debug, Default)]
pub struct IntelligentInterpreter {
    type_inferrer: TypeInferrer,
    relationship_detector: RelationshipDetector,
    semantic_analyzer: SemanticAnalyzer,
}

impl IntelligentInterpreter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Interpret a field from key-value pair.
    pub fn interpret_field(
        &self,
        key: &str,
        value: &YamlValue,
        context: &InterpretationContext,
    ) -> InterpretedField {
        let mut signals = Vec::new();
        let mut warnings = Vec::new();

        // Step 1: Detect definition style
        let style = self.detect_definition_style(key, value, &mut signals);

        // Step 2: Extract raw components based on style
        let (description, constraints, enum_values, formula, default_value, explicit_type, required, nested_props) =
            self.extract_components(value, style, context);

        // Step 3: Check for relationship
        let relationship = self.relationship_detector.detect(key, value, context);

        // Step 4: Infer type with conflict detection
        let (inferred_type, type_warnings) = if let Some(rel) = &relationship {
            (InferredType {
                primitive: if rel.cardinality == Cardinality::OneToMany {
                    PrimitiveType::Array
                } else {
                    PrimitiveType::Identifier
                },
                confidence: rel.confidence,
                signals: rel.signals.clone(),
            }, Vec::new())
        } else {
            let result = self.type_inferrer.infer_type_with_warnings(
                key,
                value,
                explicit_type.as_deref(),
                description.as_deref(),
                &enum_values,
            );
            (result.inferred_type, result.warnings)
        };
        warnings.extend(type_warnings);

        // Step 5: Classify semantics (static vs instance)
        let semantics = self.semantic_analyzer.analyze(
            key,
            &inferred_type,
            &formula,
            context,
            &mut signals,
        );

        // Build final result
        InterpretedField {
            key: key.to_string(),
            style,
            inferred_type,
            semantics,
            relationship,
            enum_values,
            default_value,
            formula,
            constraints,
            description,
            required,
            signals,
            warnings,
            nested_properties: nested_props,
        }
    }

    /// Detect the definition style.
    fn detect_definition_style(
        &self,
        key: &str,
        value: &YamlValue,
        signals: &mut Vec<InterpretationSignal>,
    ) -> DefinitionStyle {
        match value {
            YamlValue::String(s) => {
                // Check for array suffix: "Entity[]" or "string[]"
                if ARRAY_SUFFIX_RE.is_match(s) || LOWERCASE_ARRAY_RE.is_match(s) {
                    signals.push(InterpretationSignal {
                        source: "pattern".into(),
                        signal: format!("Array type notation detected: {}", s),
                        confidence: Confidence::Certain,
                    });
                    return DefinitionStyle::ArraySuffix;
                }

                // Check for arrow reference: "-> Entity"
                if ARROW_REF_RE.is_match(s) {
                    signals.push(InterpretationSignal {
                        source: "pattern".into(),
                        signal: "Arrow reference notation detected".into(),
                        confidence: Confidence::Certain,
                    });
                    return DefinitionStyle::ArrowReference;
                }

                // Check for pipe-separated enum: "a | b | c"
                if PIPE_ENUM_RE.is_match(s) {
                    signals.push(InterpretationSignal {
                        source: "pattern".into(),
                        signal: "Pipe-separated enum values detected".into(),
                        confidence: Confidence::Certain,
                    });
                    return DefinitionStyle::PipeSeparatedEnum;
                }

                // Check for inline constraint: "decimal(0..100)"
                if INLINE_CONSTRAINT_RE.is_match(s) {
                    signals.push(InterpretationSignal {
                        source: "pattern".into(),
                        signal: "Inline constraint notation detected".into(),
                        confidence: Confidence::Certain,
                    });
                    return DefinitionStyle::ConstraintInline;
                }

                // Check for natural language (contains spaces and common words)
                if s.contains(' ') && (s.contains("the ") || s.contains("a ") || s.ends_with('.')) {
                    signals.push(InterpretationSignal {
                        source: "content".into(),
                        signal: "Natural language description detected".into(),
                        confidence: Confidence::Medium,
                    });
                    return DefinitionStyle::NaturalLanguage;
                }

                // Check for dot path: "entity.attribute"
                if key.contains('.') {
                    return DefinitionStyle::FlatDotPath;
                }

                // Simple type shorthand
                DefinitionStyle::InlineShorthand
            }

            YamlValue::Mapping(map) => {
                // Check for explicit type field
                if map.contains_key(&YamlValue::String("type".into()))
                    || map.contains_key(&YamlValue::String("kind".into()))
                {
                    signals.push(InterpretationSignal {
                        source: "structure".into(),
                        signal: "Explicit type field found".into(),
                        confidence: Confidence::Certain,
                    });

                    // Check if it also has computed/formula
                    if map.contains_key(&YamlValue::String("computed".into()))
                        || map.contains_key(&YamlValue::String("formula".into()))
                        || map.contains_key(&YamlValue::String("expression".into()))
                    {
                        return DefinitionStyle::ComputedFormula;
                    }

                    return DefinitionStyle::ExplicitTyped;
                }

                // Check for relationship structure
                if map.contains_key(&YamlValue::String("target".into()))
                    || map.contains_key(&YamlValue::String("references".into()))
                {
                    signals.push(InterpretationSignal {
                        source: "structure".into(),
                        signal: "Relationship structure detected".into(),
                        confidence: Confidence::Certain,
                    });
                    return DefinitionStyle::ArrowReference;
                }

                // Nested object with other keys
                DefinitionStyle::NestedObject
            }

            YamlValue::Sequence(_) => {
                // Array of values - likely enum options
                signals.push(InterpretationSignal {
                    source: "structure".into(),
                    signal: "Array of values - treating as enum".into(),
                    confidence: Confidence::High,
                });
                DefinitionStyle::InlineShorthand
            }

            _ => DefinitionStyle::Unknown,
        }
    }

    /// Extract components from the value based on style.
    #[allow(clippy::type_complexity)]
    fn extract_components(
        &self,
        value: &YamlValue,
        style: DefinitionStyle,
        context: &InterpretationContext,
    ) -> (
        Option<String>,      // description
        FieldConstraints,    // constraints
        Option<Vec<String>>, // enum_values
        Option<String>,      // formula
        Option<serde_json::Value>, // default
        Option<String>,      // explicit_type
        bool,                // required
        Option<Vec<InterpretedField>>, // nested_properties
    ) {
        let mut description = None;
        let mut constraints = FieldConstraints::default();
        let mut enum_values = None;
        let mut formula = None;
        let mut default_value = None;
        let mut explicit_type = None;
        let mut required = false;
        let mut nested_properties = None;

        match value {
            YamlValue::String(s) => {
                match style {
                    DefinitionStyle::PipeSeparatedEnum => {
                        enum_values = Some(
                            s.split('|')
                                .map(|v| v.trim().to_string())
                                .collect(),
                        );
                    }
                    DefinitionStyle::InlineShorthand => {
                        explicit_type = Some(s.clone());
                    }
                    DefinitionStyle::NaturalLanguage => {
                        description = Some(s.clone());
                    }
                    DefinitionStyle::ArraySuffix => {
                        // Handle Entity[] (uppercase) or string[] (lowercase)
                        if let Some(caps) = ARRAY_SUFFIX_RE.captures(s) {
                            explicit_type = Some(format!("{}[]", &caps[1]));
                        } else if let Some(caps) = LOWERCASE_ARRAY_RE.captures(s) {
                            explicit_type = Some(format!("{}[]", &caps[1]));
                        }
                    }
                    DefinitionStyle::ConstraintInline => {
                        if let Some(caps) = INLINE_CONSTRAINT_RE.captures(s) {
                            explicit_type = Some(caps[1].to_string());
                            // Parse constraint part
                            let constraint_str = &caps[2];
                            // Handle enum[a,b,c]
                            if caps[1].to_lowercase() == "enum" {
                                enum_values = Some(
                                    constraint_str
                                        .split(',')
                                        .map(|v| v.trim().to_string())
                                        .collect(),
                                );
                            }
                        }
                    }
                    _ => {}
                }
            }

            YamlValue::Mapping(map) => {
                // Extract type
                if let Some(t) = map.get(&YamlValue::String("type".into())) {
                    if let YamlValue::String(ts) = t {
                        explicit_type = Some(ts.clone());
                    }
                }

                // Extract description
                if let Some(d) = map.get(&YamlValue::String("description".into())) {
                    if let YamlValue::String(ds) = d {
                        description = Some(ds.clone());
                    }
                }

                // Extract constraints
                if let Some(m) = map.get(&YamlValue::String("min".into())) {
                    constraints.min = yaml_to_json(m);
                }
                if let Some(m) = map.get(&YamlValue::String("max".into())) {
                    constraints.max = yaml_to_json(m);
                }
                if let Some(p) = map.get(&YamlValue::String("pattern".into())) {
                    if let YamlValue::String(ps) = p {
                        constraints.pattern = Some(ps.clone());
                    }
                }

                // Extract enum values
                if let Some(v) = map.get(&YamlValue::String("values".into())) {
                    if let YamlValue::Sequence(seq) = v {
                        enum_values = Some(
                            seq.iter()
                                .filter_map(|v| {
                                    if let YamlValue::String(s) = v {
                                        Some(s.clone())
                                    } else {
                                        None
                                    }
                                })
                                .collect(),
                        );
                    }
                }

                // Extract formula
                for key in ["computed", "formula", "expression"] {
                    if let Some(f) = map.get(&YamlValue::String(key.into())) {
                        if let YamlValue::String(fs) = f {
                            formula = Some(fs.clone());
                            break;
                        }
                    }
                }

                // Extract default
                if let Some(d) = map.get(&YamlValue::String("default".into())) {
                    default_value = yaml_to_json(d);
                }

                // Extract required
                if let Some(r) = map.get(&YamlValue::String("required".into())) {
                    if let YamlValue::Bool(b) = r {
                        required = *b;
                    }
                }

                // Extract and recursively parse nested properties (for object/array types)
                if let Some(props) = map.get(&YamlValue::String("properties".into())) {
                    if let YamlValue::Mapping(props_map) = props {
                        let mut nested = Vec::new();
                        for (prop_key, prop_value) in props_map {
                            if let YamlValue::String(key_str) = prop_key {
                                let nested_field = self.interpret_field(key_str, prop_value, context);
                                nested.push(nested_field);
                            }
                        }
                        if !nested.is_empty() {
                            nested_properties = Some(nested);
                        }
                    }
                }

                // Also check for 'items' (for array item types)
                if let Some(items) = map.get(&YamlValue::String("items".into())) {
                    if let YamlValue::Mapping(items_map) = items {
                        let mut nested = nested_properties.take().unwrap_or_default();
                        for (item_key, item_value) in items_map {
                            if let YamlValue::String(key_str) = item_key {
                                let nested_field = self.interpret_field(key_str, item_value, context);
                                nested.push(nested_field);
                            }
                        }
                        if !nested.is_empty() {
                            nested_properties = Some(nested);
                        }
                    }
                }
            }

            YamlValue::Sequence(seq) => {
                // Treat as enum values
                enum_values = Some(
                    seq.iter()
                        .filter_map(|v| {
                            if let YamlValue::String(s) = v {
                                Some(s.clone())
                            } else {
                                None
                            }
                        })
                        .collect(),
                );
            }

            _ => {}
        }

        (description, constraints, enum_values, formula, default_value, explicit_type, required, nested_properties)
    }
}

// =============================================================================
// Type Inferrer
// =============================================================================

#[derive(Debug, Default)]
pub struct TypeInferrer;

/// Result of type inference including any warnings.
#[derive(Debug, Clone)]
pub struct TypeInferenceResult {
    pub inferred_type: InferredType,
    pub warnings: Vec<TypeConflictWarning>,
}

impl TypeInferrer {
    /// Check what type would be inferred from name patterns alone.
    fn get_name_pattern_type(&self, key: &str) -> Option<(PrimitiveType, String)> {
        for (pattern, ptype) in NAME_PATTERNS.iter() {
            if pattern.is_match(key) {
                return Some((*ptype, pattern.as_str().to_string()));
            }
        }
        None
    }

    /// Infer type from multiple signals, returning both the type and any warnings.
    pub fn infer_type_with_warnings(
        &self,
        key: &str,
        value: &YamlValue,
        explicit_type: Option<&str>,
        description: Option<&str>,
        enum_values: &Option<Vec<String>>,
    ) -> TypeInferenceResult {
        let mut signals = Vec::new();
        let mut warnings = Vec::new();

        // Priority 1: Explicit type
        if let Some(type_str) = explicit_type {
            if let Some(prim) = parse_type_string(type_str) {
                signals.push(format!("Explicit type: {}", type_str));

                // Check if name pattern would suggest a different type
                if let Some((pattern_type, pattern_str)) = self.get_name_pattern_type(key) {
                    if pattern_type != prim {
                        warnings.push(TypeConflictWarning {
                            field: key.to_string(),
                            explicit_type: type_str.to_string(),
                            inferred_type: pattern_type.to_datatype_id().to_string(),
                            message: format!(
                                "Field '{}' has explicit type '{}' but name pattern '{}' suggests '{}'. \
                                 This may be intentional, but verify the type is correct.",
                                key, type_str, pattern_str, pattern_type.to_datatype_id()
                            ),
                        });
                    }
                }

                return TypeInferenceResult {
                    inferred_type: InferredType {
                        primitive: prim,
                        confidence: Confidence::Certain,
                        signals,
                    },
                    warnings,
                };
            }
        }

        // Priority 2: Enum values present
        if enum_values.is_some() {
            signals.push("Enum values present".to_string());
            return TypeInferenceResult {
                inferred_type: InferredType {
                    primitive: PrimitiveType::String, // Enums are string-based
                    confidence: Confidence::Certain,
                    signals,
                },
                warnings,
            };
        }

        // Priority 3: Name pattern matching
        for (pattern, ptype) in NAME_PATTERNS.iter() {
            if pattern.is_match(key) {
                signals.push(format!("Name pattern match: {}", pattern.as_str()));
                return TypeInferenceResult {
                    inferred_type: InferredType {
                        primitive: *ptype,
                        confidence: Confidence::High,
                        signals,
                    },
                    warnings,
                };
            }
        }

        // Priority 4: Value-based inference
        if let Some(prim) = infer_from_value(value) {
            signals.push("Value-based inference".to_string());
            return TypeInferenceResult {
                inferred_type: InferredType {
                    primitive: prim,
                    confidence: Confidence::Medium,
                    signals,
                },
                warnings,
            };
        }

        // Priority 5: Description-based (light NLP)
        if let Some(desc) = description {
            if let Some(prim) = infer_from_description(desc) {
                signals.push(format!("Description hint: {}", desc));
                return TypeInferenceResult {
                    inferred_type: InferredType {
                        primitive: prim,
                        confidence: Confidence::Low,
                        signals,
                    },
                    warnings,
                };
            }
        }

        // Default: String
        signals.push("Default fallback".to_string());
        TypeInferenceResult {
            inferred_type: InferredType {
                primitive: PrimitiveType::String,
                confidence: Confidence::Default,
                signals,
            },
            warnings,
        }
    }

    /// Infer type from multiple signals (legacy method for compatibility).
    pub fn infer_type(
        &self,
        key: &str,
        value: &YamlValue,
        explicit_type: Option<&str>,
        description: Option<&str>,
        enum_values: &Option<Vec<String>>,
    ) -> InferredType {
        self.infer_type_with_warnings(key, value, explicit_type, description, enum_values)
            .inferred_type
    }
}

fn parse_type_string(s: &str) -> Option<PrimitiveType> {
    let lower = s.to_lowercase();

    // Check for array type notation: "string[]", "decimal[]", "Entity[]", etc.
    if lower.ends_with("[]") {
        return Some(PrimitiveType::Array);
    }

    match lower.as_str() {
        "string" | "str" | "text" => Some(PrimitiveType::String),
        "int" | "integer" | "number" => Some(PrimitiveType::Int),
        "decimal" | "float" | "double" | "numeric" => Some(PrimitiveType::Decimal),
        "bool" | "boolean" => Some(PrimitiveType::Bool),
        "datetime" | "date" | "time" | "timestamp" => Some(PrimitiveType::Datetime),
        "id" | "identifier" | "uuid" => Some(PrimitiveType::Identifier),
        "object" | "map" | "dict" => Some(PrimitiveType::Object),
        "array" | "list" => Some(PrimitiveType::Array),
        _ => None,
    }
}

fn infer_from_value(value: &YamlValue) -> Option<PrimitiveType> {
    match value {
        YamlValue::Bool(_) => Some(PrimitiveType::Bool),
        YamlValue::Number(n) => {
            if n.is_f64() {
                Some(PrimitiveType::Decimal)
            } else {
                Some(PrimitiveType::Int)
            }
        }
        YamlValue::Sequence(_) => Some(PrimitiveType::Array),
        YamlValue::Mapping(_) => Some(PrimitiveType::Object),
        _ => None,
    }
}

fn infer_from_description(desc: &str) -> Option<PrimitiveType> {
    let lower = desc.to_lowercase();
    if lower.contains("number") || lower.contains("count") || lower.contains("quantity") {
        Some(PrimitiveType::Int)
    } else if lower.contains("amount") || lower.contains("price") || lower.contains("rate") {
        Some(PrimitiveType::Decimal)
    } else if lower.contains("yes/no") || lower.contains("true/false") || lower.contains("whether") {
        Some(PrimitiveType::Bool)
    } else if lower.contains("date") || lower.contains("time") || lower.contains("when") {
        Some(PrimitiveType::Datetime)
    } else {
        None
    }
}

// =============================================================================
// Relationship Detector
// =============================================================================

#[derive(Debug, Default)]
pub struct RelationshipDetector;

impl RelationshipDetector {
    pub fn detect(
        &self,
        key: &str,
        value: &YamlValue,
        context: &InterpretationContext,
    ) -> Option<DetectedRelationship> {
        let mut signals = Vec::new();

        // Pattern 1: Explicit relationship block
        if let YamlValue::Mapping(map) = value {
            if let Some(target) = map.get(&YamlValue::String("target".into())) {
                if let YamlValue::String(target_str) = target {
                    let cardinality = map
                        .get(&YamlValue::String("cardinality".into()))
                        .and_then(|c| {
                            if let YamlValue::String(cs) = c {
                                parse_cardinality(cs)
                            } else {
                                None
                            }
                        })
                        .unwrap_or(Cardinality::ManyToOne);

                    let optional = map
                        .get(&YamlValue::String("optional".into()))
                        .and_then(|o| if let YamlValue::Bool(b) = o { Some(*b) } else { None })
                        .unwrap_or(false);

                    signals.push("Explicit target field".to_string());
                    return Some(DetectedRelationship {
                        target: target_str.clone(),
                        cardinality,
                        optional,
                        confidence: Confidence::Certain,
                        signals,
                    });
                }
            }
        }

        // Pattern 2: Array suffix - "Entity[]"
        if let YamlValue::String(s) = value {
            if let Some(caps) = ARRAY_SUFFIX_RE.captures(s) {
                let target = caps[1].to_string();
                if context.known_entities.contains(&target) {
                    signals.push(format!("Array suffix with known entity: {}", target));
                    return Some(DetectedRelationship {
                        target,
                        cardinality: Cardinality::OneToMany,
                        optional: false,
                        confidence: Confidence::Certain,
                        signals,
                    });
                }
            }

            // Pattern 3: Arrow reference - "-> Entity"
            if let Some(caps) = ARROW_REF_RE.captures(s) {
                let target = caps[1].to_string();
                let optional = caps.get(2).is_some();
                signals.push(format!("Arrow reference to: {}", target));
                return Some(DetectedRelationship {
                    target,
                    cardinality: Cardinality::ManyToOne,
                    optional,
                    confidence: Confidence::Certain,
                    signals,
                });
            }
        }

        // Pattern 4: Key name matches known entity (plural form)
        let singular = depluralize(key);
        let pascal = to_pascal_case(&singular);
        if context.known_entities.contains(&pascal) {
            signals.push(format!("Key name matches entity: {} -> {}", key, pascal));
            return Some(DetectedRelationship {
                target: pascal,
                cardinality: if key.ends_with('s') || key.ends_with("ies") {
                    Cardinality::OneToMany
                } else {
                    Cardinality::ManyToOne
                },
                optional: false,
                confidence: Confidence::Medium,
                signals,
            });
        }

        None
    }
}

fn parse_cardinality(s: &str) -> Option<Cardinality> {
    match s.to_lowercase().replace('-', "_").as_str() {
        "one_to_one" | "1:1" => Some(Cardinality::OneToOne),
        "one_to_many" | "1:n" | "1:*" | "many" => Some(Cardinality::OneToMany),
        "many_to_one" | "n:1" | "*:1" | "one" => Some(Cardinality::ManyToOne),
        "many_to_many" | "n:n" | "*:*" => Some(Cardinality::ManyToMany),
        _ => None,
    }
}

fn depluralize(s: &str) -> String {
    // Common singular words ending in 's' that shouldn't be depluralized
    const SINGULAR_WORDS_ENDING_IN_S: &[&str] = &[
        "status", "bonus", "bus", "campus", "corpus", "focus", "genus",
        "nexus", "radius", "virus", "alias", "atlas", "basis", "crisis",
        "thesis", "analysis", "emphasis", "synopsis",
    ];

    if SINGULAR_WORDS_ENDING_IN_S.contains(&s.to_lowercase().as_str()) {
        return s.to_string();
    }

    if s.ends_with("ies") {
        format!("{}y", &s[..s.len() - 3])
    } else if s.ends_with("ses") || s.ends_with("xes") || s.ends_with("zes") ||
              s.ends_with("ches") || s.ends_with("shes") {
        // Remove "es" for words like "classes", "boxes", "quizzes", "matches", "dishes"
        s[..s.len() - 2].to_string()
    } else if s.ends_with('s') && !s.ends_with("ss") && s.len() > 2 {
        s[..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

fn to_pascal_case(s: &str) -> String {
    s.split(|c: char| c == '_' || c == '-' || c == ' ')
        .filter(|p| !p.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_uppercase().chain(chars).collect(),
            }
        })
        .collect()
}

// =============================================================================
// Semantic Analyzer (Static vs Instance)
// =============================================================================

#[derive(Debug, Default)]
pub struct SemanticAnalyzer;

impl SemanticAnalyzer {
    pub fn analyze(
        &self,
        key: &str,
        inferred_type: &InferredType,
        formula: &Option<String>,
        context: &InterpretationContext,
        signals: &mut Vec<InterpretationSignal>,
    ) -> AttributeSemantics {
        let mut static_score: f32 = 0.0;
        let mut instance_score: f32 = 0.0;

        // Signal 1: Entity naming - enhanced with definition/template detection
        if let Some(entity) = &context.current_entity {
            // Instance-type entities - runtime data, per-user/per-session values
            if entity.ends_with("Instance") || entity.ends_with("Record")
                || entity.ends_with("State") || entity.ends_with("Event")
                || entity.ends_with("Log") || entity.ends_with("Entry")
                || entity.ends_with("Session") || entity.ends_with("Request")
                || entity.ends_with("Response") || entity.ends_with("Transaction")
                || entity.ends_with("Action") || entity.ends_with("Activity")
                || entity.ends_with("Attempt") || entity.ends_with("Result")
                || entity.ends_with("Submission") || entity.ends_with("Upload")
                || entity.ends_with("Download") || entity.ends_with("Notification")
                || entity.ends_with("Message") || entity.ends_with("Comment")
                || entity.ends_with("Review") || entity.ends_with("Vote")
                || entity.ends_with("Like") || entity.ends_with("Share")
                || entity.ends_with("Click") || entity.ends_with("View")
                || entity.ends_with("Visit") || entity.ends_with("Impression")
                || entity.ends_with("Conversion") || entity.ends_with("Purchase")
                || entity.ends_with("Payment") || entity.ends_with("Refund")
                || entity.ends_with("Subscription") || entity.ends_with("Booking")
                || entity.ends_with("Reservation") || entity.ends_with("Appointment")
            {
                instance_score += Confidence::High.weight();
                signals.push(InterpretationSignal {
                    source: "entity_name".into(),
                    signal: format!("Entity {} suggests instance/runtime data", entity),
                    confidence: Confidence::High,
                });
            }
            // Definition/template-type entities - fields should be static
            else if entity.ends_with("Spec") || entity.ends_with("Template")
                || entity.ends_with("Definition") || entity.ends_with("Config")
                || entity.ends_with("Configuration") || entity.ends_with("Settings")
                || entity.ends_with("Schema") || entity.ends_with("Persona")
                || entity.ends_with("Rule") || entity.ends_with("Criteria")
                || entity.ends_with("Policy") || entity.ends_with("Constraint")
                || entity.ends_with("Model") || entity.ends_with("Type")
                || entity.ends_with("Kind") || entity.ends_with("Category")
                || entity.ends_with("Enum") || entity.ends_with("Option")
                || entity.ends_with("Choice") || entity.ends_with("Variant")
                || entity.ends_with("Blueprint") || entity.ends_with("Prototype")
                || entity.ends_with("Plan") || entity.ends_with("Tier")
                || entity.ends_with("Level") || entity.ends_with("Grade")
                || entity.ends_with("Role") || entity.ends_with("Permission")
                || entity.ends_with("Capability") || entity.ends_with("Feature")
                || entity.ends_with("Workflow") || entity.ends_with("Pipeline")
                || entity.ends_with("Stage") || entity.ends_with("Step")
                || entity.ends_with("Metric") || entity.ends_with("KPI")
                || entity.ends_with("Benchmark") || entity.ends_with("Standard")
            {
                static_score += Confidence::High.weight();
                signals.push(InterpretationSignal {
                    source: "entity_name".into(),
                    signal: format!("Entity {} is a definition/template - fields are static", entity),
                    confidence: Confidence::High,
                });
            }
            // Scenario, Phase, Competency are also definition entities
            else if entity == "Scenario" || entity == "Phase" || entity == "Competency"
                || entity == "Observable" || entity == "AgentPersona"
                || entity == "Skill" || entity == "Trait" || entity == "Attribute"
                || entity == "Dimension" || entity == "Factor" || entity == "Indicator"
                || entity == "Rubric" || entity == "Criterion" || entity == "Assessment"
            {
                static_score += Confidence::Medium.weight();
                signals.push(InterpretationSignal {
                    source: "entity_name".into(),
                    signal: format!("Entity {} is typically a definition entity", entity),
                    confidence: Confidence::Medium,
                });
            }
        }

        // Signal 2: Attribute naming for instance indicators
        if key.ends_with("_at") || key.ends_with("_by")
            || key.starts_with("created") || key.starts_with("updated")
        {
            instance_score += Confidence::High.weight();
            signals.push(InterpretationSignal {
                source: "name_pattern".into(),
                signal: "Timestamp/audit pattern suggests instance".into(),
                confidence: Confidence::High,
            });
        }

        // Signal 3: Attribute naming for static indicators
        if key.starts_with("template_") || key.starts_with("default_") || key.starts_with("base_") {
            static_score += Confidence::High.weight();
            signals.push(InterpretationSignal {
                source: "name_pattern".into(),
                signal: "Template/default prefix suggests static".into(),
                confidence: Confidence::High,
            });
        }

        // Signal 4: Computed/formula implies instance
        if formula.is_some() {
            instance_score += Confidence::High.weight();
            signals.push(InterpretationSignal {
                source: "formula".into(),
                signal: "Computed field suggests instance".into(),
                confidence: Confidence::High,
            });
        }

        // Signal 5: Referenced in formulas implies instance
        if context.formula_references.contains(key) {
            instance_score += Confidence::Medium.weight();
            signals.push(InterpretationSignal {
                source: "formula_ref".into(),
                signal: "Referenced in formula suggests instance".into(),
                confidence: Confidence::Medium,
            });
        }

        // Signal 6: Enum definition (not value) is static
        let key_lower = key.to_lowercase();
        if inferred_type.primitive == PrimitiveType::String
            && (key_lower.contains("type") || key_lower.contains("kind"))
        {
            static_score += Confidence::Medium.weight();
            signals.push(InterpretationSignal {
                source: "type_field".into(),
                signal: "Type/kind field often static".into(),
                confidence: Confidence::Medium,
            });
        }

        // Aggregate
        if static_score > instance_score * 1.5 {
            AttributeSemantics::Static { confidence: static_score }
        } else {
            // Default to instance (safer for runtime values)
            AttributeSemantics::Instance {
                confidence: instance_score.max(0.5),
            }
        }
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

fn yaml_to_json(value: &YamlValue) -> Option<serde_json::Value> {
    match value {
        YamlValue::Null => Some(serde_json::Value::Null),
        YamlValue::Bool(b) => Some(serde_json::Value::Bool(*b)),
        YamlValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                Some(serde_json::Value::Number(i.into()))
            } else if let Some(f) = n.as_f64() {
                serde_json::Number::from_f64(f).map(serde_json::Value::Number)
            } else {
                None
            }
        }
        YamlValue::String(s) => Some(serde_json::Value::String(s.clone())),
        YamlValue::Sequence(seq) => {
            let arr: Vec<_> = seq.iter().filter_map(yaml_to_json).collect();
            Some(serde_json::Value::Array(arr))
        }
        YamlValue::Mapping(map) => {
            let obj: serde_json::Map<String, serde_json::Value> = map
                .iter()
                .filter_map(|(k, v)| {
                    if let YamlValue::String(ks) = k {
                        yaml_to_json(v).map(|jv| (ks.clone(), jv))
                    } else {
                        None
                    }
                })
                .collect();
            Some(serde_json::Value::Object(obj))
        }
        _ => None,
    }
}

/// Extract variable references from a formula.
pub fn extract_formula_references(formula: &str) -> Vec<String> {
    let mut refs = Vec::new();

    // JSON Logic var references
    for cap in JSON_LOGIC_VAR_RE.captures_iter(formula) {
        refs.push(cap[1].to_string());
    }

    // Template variable references
    for cap in TEMPLATE_VAR_RE.captures_iter(formula) {
        if let Some(m) = cap.get(1) {
            refs.push(m.as_str().to_string());
        } else if let Some(m) = cap.get(2) {
            refs.push(m.as_str().to_string());
        }
    }

    refs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_definition_style_detection() {
        let interp = IntelligentInterpreter::new();
        let ctx = InterpretationContext::default();

        // Array suffix
        let field = interp.interpret_field(
            "users",
            &YamlValue::String("User[]".into()),
            &ctx,
        );
        assert_eq!(field.style, DefinitionStyle::ArraySuffix);

        // Pipe enum
        let field = interp.interpret_field(
            "status",
            &YamlValue::String("pending | active | done".into()),
            &ctx,
        );
        assert_eq!(field.style, DefinitionStyle::PipeSeparatedEnum);
        assert!(field.enum_values.is_some());
    }

    #[test]
    fn test_type_inference() {
        let inferrer = TypeInferrer;

        // Name pattern: *_count -> Int
        let result = inferrer.infer_type("user_count", &YamlValue::Null, None, None, &None);
        assert_eq!(result.primitive, PrimitiveType::Int);
        assert_eq!(result.confidence, Confidence::High);

        // Name pattern: is_* -> Bool
        let result = inferrer.infer_type("is_active", &YamlValue::Null, None, None, &None);
        assert_eq!(result.primitive, PrimitiveType::Bool);

        // Name pattern: *_at -> Datetime
        let result = inferrer.infer_type("created_at", &YamlValue::Null, None, None, &None);
        assert_eq!(result.primitive, PrimitiveType::Datetime);
    }

    #[test]
    fn test_relationship_detection() {
        let detector = RelationshipDetector;
        let mut ctx = InterpretationContext::default();
        ctx.known_entities.insert("User".to_string());

        // Array suffix with known entity
        let rel = detector.detect(
            "users",
            &YamlValue::String("User[]".into()),
            &ctx,
        );
        assert!(rel.is_some());
        let rel = rel.unwrap();
        assert_eq!(rel.target, "User");
        assert_eq!(rel.cardinality, Cardinality::OneToMany);

        // Arrow reference
        let rel = detector.detect(
            "owner",
            &YamlValue::String("-> User".into()),
            &ctx,
        );
        assert!(rel.is_some());
        let rel = rel.unwrap();
        assert_eq!(rel.target, "User");
        assert_eq!(rel.cardinality, Cardinality::ManyToOne);
    }

    #[test]
    fn test_formula_reference_extraction() {
        let refs = extract_formula_references(r#"{"*": [{"var": "price"}, {"var": "quantity"}]}"#);
        assert!(refs.contains(&"price".to_string()));
        assert!(refs.contains(&"quantity".to_string()));

        let refs = extract_formula_references("${user.name} - {{order.total}}");
        assert!(refs.contains(&"user.name".to_string()));
        assert!(refs.contains(&"order.total".to_string()));
    }

    #[test]
    fn test_depluralize() {
        assert_eq!(depluralize("users"), "user");
        assert_eq!(depluralize("categories"), "category");
        assert_eq!(depluralize("status"), "status"); // doesn't end in 's' pattern
    }

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("user_profile"), "UserProfile");
        assert_eq!(to_pascal_case("order-item"), "OrderItem");
        assert_eq!(to_pascal_case("simple"), "Simple");
    }

    #[test]
    fn test_array_type_detection() {
        let interp = IntelligentInterpreter::new();
        let ctx = InterpretationContext::default();

        // Lowercase array type: string[]
        let field = interp.interpret_field(
            "tags",
            &YamlValue::String("string[]".into()),
            &ctx,
        );
        assert_eq!(field.style, DefinitionStyle::ArraySuffix);
        assert_eq!(field.inferred_type.primitive, PrimitiveType::Array);
        assert_eq!(field.inferred_type.confidence, Confidence::Certain);

        // Other lowercase array types
        let field = interp.interpret_field(
            "scores",
            &YamlValue::String("decimal[]".into()),
            &ctx,
        );
        assert_eq!(field.inferred_type.primitive, PrimitiveType::Array);
        assert_eq!(field.inferred_type.confidence, Confidence::Certain);

        // Entity array type: User[]
        let field = interp.interpret_field(
            "users",
            &YamlValue::String("User[]".into()),
            &ctx,
        );
        assert_eq!(field.style, DefinitionStyle::ArraySuffix);
        assert_eq!(field.inferred_type.primitive, PrimitiveType::Array);
    }

    #[test]
    fn test_parse_type_string_arrays() {
        assert_eq!(parse_type_string("string[]"), Some(PrimitiveType::Array));
        assert_eq!(parse_type_string("decimal[]"), Some(PrimitiveType::Array));
        assert_eq!(parse_type_string("User[]"), Some(PrimitiveType::Array));
        assert_eq!(parse_type_string("int[]"), Some(PrimitiveType::Array));
        // Non-array types still work
        assert_eq!(parse_type_string("string"), Some(PrimitiveType::String));
        assert_eq!(parse_type_string("decimal"), Some(PrimitiveType::Decimal));
    }

    #[test]
    fn test_timestamp_pattern() {
        let inferrer = TypeInferrer;

        // Exact match for 'timestamp' should infer Datetime
        let result = inferrer.infer_type("timestamp", &YamlValue::Null, None, None, &None);
        assert_eq!(result.primitive, PrimitiveType::Datetime);
        assert_eq!(result.confidence, Confidence::High);
    }

    #[test]
    fn test_updated_suffix_pattern() {
        let inferrer = TypeInferrer;

        // *_updated suffix should infer Datetime
        let result = inferrer.infer_type("last_updated", &YamlValue::Null, None, None, &None);
        assert_eq!(result.primitive, PrimitiveType::Datetime);
        assert_eq!(result.confidence, Confidence::High);

        let result = inferrer.infer_type("profile_updated", &YamlValue::Null, None, None, &None);
        assert_eq!(result.primitive, PrimitiveType::Datetime);
        assert_eq!(result.confidence, Confidence::High);
    }

    #[test]
    fn test_type_conflict_warning() {
        let inferrer = TypeInferrer;

        // Explicit type decimal conflicts with *_at pattern suggesting datetime
        let result = inferrer.infer_type_with_warnings(
            "last_activity_at",
            &YamlValue::Null,
            Some("decimal"),
            None,
            &None,
        );

        // Type should be decimal (explicit wins)
        assert_eq!(result.inferred_type.primitive, PrimitiveType::Decimal);
        assert_eq!(result.inferred_type.confidence, Confidence::Certain);

        // But there should be a warning about the conflict
        assert!(!result.warnings.is_empty());
        assert!(result.warnings[0].message.contains("datetime"));
        assert!(result.warnings[0].explicit_type == "decimal");
    }

    #[test]
    fn test_entity_level_static_classification() {
        let interp = IntelligentInterpreter::new();

        // Definition entity (ends with Spec) should default fields to static
        let mut ctx = InterpretationContext::default();
        ctx.current_entity = Some("PhaseSpec".to_string());

        let field = interp.interpret_field(
            "name",
            &YamlValue::String("string".into()),
            &ctx,
        );

        // Should be classified as static because entity is a Spec
        assert!(field.semantics.is_static());
    }

    #[test]
    fn test_entity_level_instance_classification() {
        let interp = IntelligentInterpreter::new();

        // State entity should default fields to instance
        let mut ctx = InterpretationContext::default();
        ctx.current_entity = Some("SessionState".to_string());

        let field = interp.interpret_field(
            "name",
            &YamlValue::String("string".into()),
            &ctx,
        );

        // Should be classified as instance because entity is a State
        assert!(field.semantics.is_instance());
    }

    #[test]
    fn test_nested_property_parsing() {
        let interp = IntelligentInterpreter::new();
        let ctx = InterpretationContext::default();

        // Create a mapping with type: object and properties
        let mut props = serde_yaml::Mapping::new();
        props.insert(
            YamlValue::String("difficulty".into()),
            YamlValue::String("easy | medium | hard".into()),
        );
        // Use a nested object without explicit type to test name pattern inference
        let mut timestamp_map = serde_yaml::Mapping::new();
        timestamp_map.insert(
            YamlValue::String("description".into()),
            YamlValue::String("When the session started".into()),
        );
        props.insert(
            YamlValue::String("started_at".into()),
            YamlValue::Mapping(timestamp_map),
        );

        let mut map = serde_yaml::Mapping::new();
        map.insert(YamlValue::String("type".into()), YamlValue::String("object".into()));
        map.insert(YamlValue::String("properties".into()), YamlValue::Mapping(props));

        let field = interp.interpret_field(
            "preferences",
            &YamlValue::Mapping(map),
            &ctx,
        );

        // Should have nested properties
        assert!(field.nested_properties.is_some());
        let nested = field.nested_properties.unwrap();
        assert_eq!(nested.len(), 2);

        // Find the difficulty field and check it has enum values
        let difficulty_field = nested.iter().find(|f| f.key == "difficulty").unwrap();
        assert!(difficulty_field.enum_values.is_some());
        assert_eq!(difficulty_field.enum_values.as_ref().unwrap(), &vec!["easy", "medium", "hard"]);

        // Find the started_at field - without explicit type, name pattern *_at should infer datetime
        let started_at_field = nested.iter().find(|f| f.key == "started_at").unwrap();
        assert_eq!(started_at_field.inferred_type.primitive, PrimitiveType::Datetime);
    }

    #[test]
    fn test_nested_explicit_type_wins() {
        let interp = IntelligentInterpreter::new();
        let ctx = InterpretationContext::default();

        // Create a nested property with explicit type that conflicts with name pattern
        let mut props = serde_yaml::Mapping::new();
        props.insert(
            YamlValue::String("last_activity_at".into()),
            YamlValue::String("decimal".into()), // Explicit decimal overrides *_at datetime
        );

        let mut map = serde_yaml::Mapping::new();
        map.insert(YamlValue::String("type".into()), YamlValue::String("object".into()));
        map.insert(YamlValue::String("properties".into()), YamlValue::Mapping(props));

        let field = interp.interpret_field(
            "timing",
            &YamlValue::Mapping(map),
            &ctx,
        );

        // Should have nested properties
        assert!(field.nested_properties.is_some());
        let nested = field.nested_properties.unwrap();
        assert_eq!(nested.len(), 1);

        // Explicit type should win
        let activity_field = nested.iter().find(|f| f.key == "last_activity_at").unwrap();
        assert_eq!(activity_field.inferred_type.primitive, PrimitiveType::Decimal);

        // But there should be a warning about the conflict
        assert!(!activity_field.warnings.is_empty());
    }

    #[test]
    fn test_first_name_last_name_are_strings() {
        let inferrer = TypeInferrer;

        // first_name should be String, NOT Datetime (regression test for pattern priority bug)
        let result = inferrer.infer_type("first_name", &YamlValue::Null, None, None, &None);
        assert_eq!(result.primitive, PrimitiveType::String, "first_name should be String, not Datetime");

        // last_name should be String, NOT Datetime
        let result = inferrer.infer_type("last_name", &YamlValue::Null, None, None, &None);
        assert_eq!(result.primitive, PrimitiveType::String, "last_name should be String, not Datetime");

        // But last_login, first_visit, last_seen should still be Datetime (specific patterns)
        let result = inferrer.infer_type("last_login", &YamlValue::Null, None, None, &None);
        assert_eq!(result.primitive, PrimitiveType::Datetime, "last_login should be Datetime");

        let result = inferrer.infer_type("first_visit", &YamlValue::Null, None, None, &None);
        assert_eq!(result.primitive, PrimitiveType::Datetime, "first_visit should be Datetime");

        let result = inferrer.infer_type("last_seen", &YamlValue::Null, None, None, &None);
        assert_eq!(result.primitive, PrimitiveType::Datetime, "last_seen should be Datetime");

        let result = inferrer.infer_type("first_login", &YamlValue::Null, None, None, &None);
        assert_eq!(result.primitive, PrimitiveType::Datetime, "first_login should be Datetime");
    }

    #[test]
    fn test_middle_name_full_name_are_strings() {
        let inferrer = TypeInferrer;

        // These should all be String
        let result = inferrer.infer_type("middle_name", &YamlValue::Null, None, None, &None);
        assert_eq!(result.primitive, PrimitiveType::String);

        let result = inferrer.infer_type("full_name", &YamlValue::Null, None, None, &None);
        assert_eq!(result.primitive, PrimitiveType::String);

        let result = inferrer.infer_type("display_name", &YamlValue::Null, None, None, &None);
        assert_eq!(result.primitive, PrimitiveType::String);
    }
}
