package com.gemwallet.android.testkit

import android.content.Context
import com.wallet.core.primitives.Chain
import uniffi.gemstone.GemImportType
import uniffi.gemstone.GemKeystore
import java.io.File

const val KEYSTORE_TEST_PASSWORD = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f"
const val KEYSTORE_TEST_ETH_ADDRESS = "0x5a8f70b44aFa00Cb70615D9c9CCb9A24933ED2D3"

fun includeGemstoneLibs() {
    System.loadLibrary("gemstone")
}

fun gemstoneTestAddressForPrivateKey(context: Context, chain: Chain, value: String): String {
    return GemKeystore(gemstoneTestBaseDir(context)).use { keystore ->
        keystore.previewImport(GemImportType.PrivateKey(value = value, chain = chain.string)).accounts.first().address
    }
}

private fun gemstoneTestBaseDir(context: Context): String {
    val dir = File(context.cacheDir, "gemstone-test-keystore")
    dir.mkdirs()
    return dir.absolutePath
}
