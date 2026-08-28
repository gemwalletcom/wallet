package com.gemwallet.android.data.repositories.gemstone

import com.gemwallet.android.data.service.store.LocalStore
import uniffi.gemstone.GemFileStore

class GemstoneFileStore(
    private val localStore: LocalStore,
) : GemFileStore {
    override fun saveFile(data: ByteArray, extension: String): String = localStore.save(data, extension)

    override fun saveNamedFile(data: ByteArray, fileName: String): String = localStore.saveNamed(data, fileName)

    override fun exists(fileName: String): Boolean = localStore.exists(fileName)

    override fun path(fileName: String): String = localStore.path(fileName)

    override fun remove(fileName: String) = localStore.remove(fileName)
}
