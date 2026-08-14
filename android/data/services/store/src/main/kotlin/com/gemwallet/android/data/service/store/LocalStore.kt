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
        File(context.filesDir, fileName).writeBytes(data)
        return fileName
    }

    fun remove(fileName: String?): Boolean {
        if (fileName.isNullOrEmpty()) return true
        val file = File(context.filesDir, fileName)
        return !file.exists() || file.delete()
    }
}
