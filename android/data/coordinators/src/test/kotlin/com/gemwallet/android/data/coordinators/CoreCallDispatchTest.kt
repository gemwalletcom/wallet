package com.gemwallet.android.data.coordinators

import java.io.File
import org.junit.Assert.assertEquals
import org.junit.Test

/**
 * Core reaches back into the app through synchronous store callbacks, and several of those read
 * Room. UniFFI polls the Rust future on the calling thread, so a Core call made from the main
 * dispatcher lands the read on main and Room throws before any work happens. Both known instances
 * of that bug (fiat quotes, confirm fee assets) reached Core from a coordinator that left the
 * dispatch to its caller.
 *
 * A coordinator that calls a Core service therefore dispatches itself, rather than trusting every
 * present and future caller to remember.
 */
class CoreCallDispatchTest {

    /**
     * Calls into pure Core rules: no store, no network, no clock. Core keeps those synchronous, so
     * they cannot be wrapped, and they are safe to run anywhere. Anything that touches a store is
     * `async` in Core and must not appear here.
     */
    private val pureRuleOnly = setOf(
        "GetBannerContentImpl.kt",
        "GetShowWelcomeBannerImpl.kt",
        "PerpetualCandlesImpl.kt",
    )

    @Test
    fun `every coordinator that calls a Core service dispatches off the caller thread`() {
        val offenders = coordinatorSources()
            .filter { it.name !in pureRuleOnly }
            .filter { it.callsCoreService() }
            .filterNot { it.readText().contains("Dispatchers.IO") }
            .map { it.name }
            .sorted()

        assertEquals(
            "These coordinators call a Core service without moving off the caller's thread. " +
                "Wrap the call in withContext(Dispatchers.IO), or flowOn(Dispatchers.IO) for a Flow. " +
                "If the call is only a pure Core rule, add it to pureRuleOnly with that stated.",
            emptyList<String>(),
            offenders,
        )
    }

    @Test
    fun `the pure rule allowlist has no stale entries`() {
        val present = coordinatorSources().map { it.name }.toSet()
        assertEquals(emptySet<String>(), pureRuleOnly - present)
    }

    private fun coordinatorSources(): List<File> {
        val root = File("src/main/kotlin/com/gemwallet/android/data/coordinators")
        check(root.isDirectory) { "coordinator sources not found at ${root.absolutePath}" }
        return root.walkTopDown()
            .filter { it.isFile && it.extension == "kt" }
            .filterNot { it.parentFile.name == "di" }
            .toList()
    }

    /** Injects a `Gem…Service` and invokes something on it, rather than merely naming a Core type. */
    private fun File.callsCoreService(): Boolean {
        val source = readText()
        val injected = INJECTED_SERVICE.findAll(source).map { it.groupValues[1] }.toSet()
        return injected.any { Regex("""\b$it\.\w""").containsMatchIn(source) }
    }

    private companion object {
        val INJECTED_SERVICE = Regex("""private val (\w+): Gem\w*Service\w*""")
    }
}
