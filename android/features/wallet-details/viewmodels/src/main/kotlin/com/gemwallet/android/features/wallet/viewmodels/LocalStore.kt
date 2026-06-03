package com.gemwallet.android.features.wallet.viewmodels

import android.content.Context
import dagger.hilt.android.qualifiers.ApplicationContext
import java.io.File
import java.util.UUID
import javax.inject.Inject

class LocalStore @Inject constructor(
    @ApplicationContext private val context: Context,
) {

    fun save(data: ByteArray, extension: String): String {
        val fileName = "${UUID.randomUUID()}.$extension"
        File(context.filesDir, fileName).writeBytes(data)
        return fileName
    }

    fun remove(fileName: String?) {
        if (fileName.isNullOrEmpty()) {
            return
        }
        File(context.filesDir, fileName).takeIf { it.exists() }?.delete()
    }
}
