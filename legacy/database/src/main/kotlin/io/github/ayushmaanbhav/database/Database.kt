package io.github.ayushmaanbhav.database

import org.springframework.boot.CommandLineRunner
import org.springframework.boot.autoconfigure.SpringBootApplication
import org.springframework.boot.runApplication

@SpringBootApplication
class Database : CommandLineRunner {
    override fun run(vararg args: String) {}

    companion object {
        @JvmStatic
        fun main(args: Array<String>) {
            runApplication<Database>(*args)
        }
    }
}
