package com.gemwallet.android

import org.junit.Assert.assertTrue
import org.junit.Test

class RootCheckerTest {
    @Test
    fun findsSuOnPath() {
        val checker = RootChecker(
            path = "/vendor/bin:/custom/bin",
            exists = { it == "/custom/bin/su" },
            readLines = { emptyList() },
        )
        assertTrue(checker.isDeviceRooted())
    }

    @Test
    fun findsModernRootArtifacts() {
        listOf("/debug_ramdisk/su", "/data/adb/ksu", "/data/adb/ap").forEach { artifact ->
            val checker = RootChecker(path = null, exists = { it == artifact }, readLines = { emptyList() })
            assertTrue(artifact, checker.isDeviceRooted())
        }
    }

    @Test
    fun findsInjectedLibraries() {
        listOf("/data/local/tmp/frida-agent-64.so", "/data/adb/modules/zygisk/libzygisk.so").forEach { library ->
            val checker = RootChecker(
                path = null,
                exists = { false },
                readLines = { if (it == "/proc/self/maps") listOf("7000-8000 r-xp 00000000 00:00 0 $library") else emptyList() },
            )
            assertTrue(library, checker.isDeviceRooted())
        }
    }

    @Test
    fun findsRootMounts() {
        val checker = RootChecker(
            path = null,
            exists = { false },
            readLines = { if (it == "/proc/self/mountinfo") listOf("100 1 0:1 /data/adb/modules/example /system/lib rw - tmpfs tmpfs rw") else emptyList() },
        )
        assertTrue(checker.isDeviceRooted())
    }
}
