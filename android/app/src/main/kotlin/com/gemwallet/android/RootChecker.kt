package com.gemwallet.android

import java.io.File
import java.io.IOException

internal class RootChecker(
    private val path: String? = System.getenv("PATH"),
    private val exists: (String) -> Boolean = { File(it).exists() },
    private val readLines: (String) -> List<String> = { File(it).readLines() },
) {
    fun isDeviceRooted(): Boolean {
        val rootPaths = listOf(
            "/system/app/Superuser.apk",
            "/sbin/su",
            "/system/bin/su",
            "/system/xbin/su",
            "/data/local/xbin/su",
            "/data/local/bin/su",
            "/system/sd/xbin/su",
            "/system/bin/failsafe/su",
            "/data/local/su",
            "/debug_ramdisk/su",
            "/sbin/.magisk",
            "/debug_ramdisk/.magisk",
            "/data/adb/magisk",
            "/data/adb/ksu",
            "/data/adb/ap",
        )
        val pathCandidates = path.orEmpty().split(File.pathSeparator)
            .filter { it.startsWith("/") }
            .map { "${it.trimEnd('/')}/su" }

        return (rootPaths + pathCandidates).any(::pathExists) ||
            hasProcessArtifact("/proc/self/mountinfo") { path ->
                rootMountPaths.any { path == it || path.startsWith("$it/") }
            } ||
            hasProcessArtifact("/proc/self/maps") { path ->
                val name = path.substringAfterLast('/').lowercase().removePrefix("lib")
                path.startsWith("/") && name.endsWith(".so") && injectedLibraryPrefixes.any(name::startsWith)
            }
    }

    private fun pathExists(path: String): Boolean = try {
        exists(path)
    } catch (_: SecurityException) {
        false
    }

    private fun hasProcessArtifact(path: String, matches: (String) -> Boolean): Boolean = try {
        readLines(path).any { line -> line.split(' ', '\t').any(matches) }
    } catch (_: IOException) {
        false
    } catch (_: SecurityException) {
        false
    }

    private companion object {
        val rootMountPaths = listOf(
            "/data/adb/magisk",
            "/data/adb/ksu",
            "/data/adb/ap",
            "/data/adb/modules",
            "/sbin/.magisk",
            "/debug_ramdisk/.magisk",
        )
        val injectedLibraryPrefixes = listOf("frida-agent", "frida-gadget", "zygisk")
    }
}
