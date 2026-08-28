package com.gemwallet.android.data.service.store

import android.content.Context
import dagger.hilt.android.qualifiers.ApplicationContext
import java.io.File
import java.util.UUID
import javax.inject.Inject

class LocalStore @Inject constructor(
    @param:ApplicationContext private val context: Context,
) {
    fun save(data: ByteArray, extension: String): String {
        val fileName = "${UUID.randomUUID()}.$extension"
        file(fileName).writeBytes(data)
        return fileName
    }

    fun saveNamed(data: ByteArray, fileName: String): String {
        val file = file(fileName)
        file.writeBytes(data)
        return file.absolutePath
    }

    fun exists(fileName: String): Boolean = file(fileName).exists()

    fun path(fileName: String): String = file(fileName).absolutePath

    fun remove(fileName: String?) {
        if (fileName.isNullOrEmpty()) {
            return
        }
        file(fileName).takeIf { it.exists() }?.delete()
    }

    private fun file(fileName: String) = File(context.filesDir, fileName)
}
