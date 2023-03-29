package io.github.ayushmaanbhav.common.exception;

import lombok.NonNull;

public class NonRetryableException extends Exception {
    public NonRetryableException(@NonNull String message) {
        super(message);
    }
}
