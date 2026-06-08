package wallet.android.app.import_wallet

import androidx.test.core.app.ApplicationProvider
import com.gemwallet.android.data.repositories.wallets.PhraseAddressImportWalletService
import com.gemwallet.android.testkit.gemstoneTestAddressForPrivateKey
import com.gemwallet.android.testkit.includeGemstoneLibs
import com.wallet.core.primitives.Chain
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.encodePrivateKey

class TestDecodePrivateKey {

    private val testBase58Key = "4ha2npeRkDXipjgGJ3L5LhZ9TK9dRjP2yktydkFBhAzXj3N8ytpYyTS24kxcYGEefy4WKWRcog2zSPvpPZoGmxCC"
    private val context = ApplicationProvider.getApplicationContext<android.content.Context>()

    companion object {
        init {
            includeGemstoneLibs()
        }
    }

    @Test
    fun testBase58Decode() {
        val base58 = "DTJi5pMtSKZHdkLX4wxwvjGjf2xwXx1LSuuUZhugYWDV"
        val key = PhraseAddressImportWalletService.decodePrivateKey(chain = Chain.Solana, testBase58Key)
        assertEquals("JSTURBrew3zGaJjtk7qcvd7gapeExX3GC7DiQBaCKzU", gemstoneTestAddressForPrivateKey(context, Chain.Solana, testBase58Key))
        assertEquals(base58, encodePrivateKey(Chain.Solana.string, key))

        val hex = "0x30df0ffc2b43717f4653c2a1e827e9dfb3d9364e019cc60092496cd4997d5d6e"
        val key2 = PhraseAddressImportWalletService.decodePrivateKey(chain = Chain.Ethereum, hex)
        assertEquals("0x4ce31c0b2114abe61Ac123E1E6254E961C18D10B", gemstoneTestAddressForPrivateKey(context, Chain.Ethereum, hex))
        assertEquals(hex, encodePrivateKey(Chain.Ethereum.string, key2))
    }

    @Test
    fun testImportStellarKey() {
        val base32Key = "SA6XNHUKMW4QAKSHB2NOZ4SYP34ERYVAWSBTEDREYSJ2LEJ5LFHLTIRJ"
        val key = PhraseAddressImportWalletService.decodePrivateKey(Chain.Stellar, base32Key)
        assertEquals("GADB4BDKTOE36L6QN2JLIPNNJ7EZPSY5BIVKWXLWYZLIPXNQWIRQQZKT", gemstoneTestAddressForPrivateKey(context, Chain.Stellar, base32Key))
        assertEquals("0x3d769e8a65b9002a470e9aecf2587ef848e2a0b483320e24c493a5913d594eb9", encodePrivateKey(Chain.Stellar.string, key))
    }
}
