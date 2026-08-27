package com.gemwallet.android.data.repositories.gemstone

import com.gemwallet.android.data.service.store.LocalStore
import uniffi.gemstone.GemFileStore

class GemstoneFileStore(
    private val localStore: LocalStore,
) : GemFileStore {
    override fun save(data: ByteArray, extension: String): String = localStore.save(data, extension)

    override fun remove(fileName: String) = localStore.remove(fileName)
}
