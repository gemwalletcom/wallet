package com.gemwallet.android.data.password

import android.content.Context
import android.content.SharedPreferences
import android.util.Xml
import org.xmlpull.v1.XmlPullParser
import java.io.File
import java.nio.file.Files
import java.nio.file.NoSuchFileException

internal val secureStorageLock = Any()

internal fun Context.securePreferences(name: String): SharedPreferences = synchronized(secureStorageLock) {
    val file = File(applicationInfo.dataDir, "shared_prefs/$name.xml")
    val isEmpty = isEmptyPreferences(File("${file.path}.bak")) ?: isEmptyPreferences(file) ?: true
    val preferences = getSharedPreferences(name, Context.MODE_PRIVATE)
    check(preferences.all.isEmpty() == isEmpty) { "Secure preferences could not be loaded" }
    preferences
}

private fun isEmptyPreferences(file: File): Boolean? {
    val input = try {
        Files.newInputStream(file.toPath())
    } catch (_: NoSuchFileException) {
        return null
    }
    return input.use {
        val parser = Xml.newPullParser().apply { setInput(it, "UTF-8") }
        check(parser.nextTag() == XmlPullParser.START_TAG && parser.name == "map") { "Invalid secure preferences" }
        parser.nextTag() == XmlPullParser.END_TAG
    }
}
