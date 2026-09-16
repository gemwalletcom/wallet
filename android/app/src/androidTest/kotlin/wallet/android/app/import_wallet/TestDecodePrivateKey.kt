package wallet.android.app.import_wallet

import androidx.test.core.app.ApplicationProvider
import com.gemwallet.android.testkit.gemstoneTestAddressForPrivateKey
import com.gemwallet.android.testkit.includeGemstoneLibs
import com.wallet.core.primitives.Chain
import org.junit.Assert.assertEquals
import org.junit.Test

class TestDecodePrivateKey {

    private val context = ApplicationProvider.getApplicationContext<android.content.Context>()

    companion object {
        init {
            includeGemstoneLibs()
        }
    }

    @Test
    fun testAddressFromBase58AndHexKeys() {
        val base58Key = "4ha2npeRkDXipjgGJ3L5LhZ9TK9dRjP2yktydkFBhAzXj3N8ytpYyTS24kxcYGEefy4WKWRcog2zSPvpPZoGmxCC"
        assertEquals("JSTURBrew3zGaJjtk7qcvd7gapeExX3GC7DiQBaCKzU", gemstoneTestAddressForPrivateKey(context, Chain.Solana, base58Key))

        val hexKey = "0x30df0ffc2b43717f4653c2a1e827e9dfb3d9364e019cc60092496cd4997d5d6e"
        assertEquals("0x4ce31c0b2114abe61Ac123E1E6254E961C18D10B", gemstoneTestAddressForPrivateKey(context, Chain.Ethereum, hexKey))
    }

    @Test
    fun testAddressFromStellarKey() {
        val base32Key = "SA6XNHUKMW4QAKSHB2NOZ4SYP34ERYVAWSBTEDREYSJ2LEJ5LFHLTIRJ"

        assertEquals("GADB4BDKTOE36L6QN2JLIPNNJ7EZPSY5BIVKWXLWYZLIPXNQWIRQQZKT", gemstoneTestAddressForPrivateKey(context, Chain.Stellar, base32Key))
    }
}
