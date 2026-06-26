package com.gemwallet.android.blockchain.operators.gemstone

import androidx.test.ext.junit.runners.AndroidJUnit4
import com.gemwallet.android.ext.checksumAddress
import com.gemwallet.android.testkit.includeGemstoneLibs
import com.wallet.core.primitives.Chain
import junit.framework.TestCase.assertEquals
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class ChecksumAddressTest {

    companion object {
        init {
            includeGemstoneLibs()
        }
    }

    @Test
    fun checksumAddress_checksumsEvmAddressesAndPassesNonEvmThrough() {
        val lowercase = "0x5aaeb6053f3e94c9b9a09f33669435e7ef1beaed"
        val checksummed = "0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed"
        val bitcoinAddress = "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh"

        assertEquals(checksummed, Chain.Ethereum.checksumAddress(lowercase))
        assertEquals(checksummed, Chain.Ethereum.checksumAddress(" \n$lowercase\r "))
        assertEquals(checksummed, Chain.SmartChain.checksumAddress(lowercase))
        assertEquals(checksummed, Chain.Ethereum.checksumAddress(checksummed))
        assertEquals(bitcoinAddress, Chain.Bitcoin.checksumAddress(" \n$bitcoinAddress\r "))
    }
}
