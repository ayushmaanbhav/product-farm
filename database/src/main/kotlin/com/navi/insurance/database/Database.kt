package com.navi.insurance.database

import org.springframework.boot.CommandLineRunner
import org.springframework.boot.autoconfigure.SpringBootApplication
import org.springframework.boot.autoconfigure.orm.jpa.HibernateJpaAutoConfiguration
import org.springframework.boot.runApplication

@SpringBootApplication(exclude = [HibernateJpaAutoConfiguration::class])
class Database : CommandLineRunner {
    override fun run(vararg args: String) {}

    companion object {
        @JvmStatic
        fun main(args: Array<String>) {
            runApplication<Database>(*args)
        }
    }
}
