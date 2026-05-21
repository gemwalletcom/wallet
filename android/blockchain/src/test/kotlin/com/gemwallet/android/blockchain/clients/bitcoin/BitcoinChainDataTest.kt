package com.gemwallet.android.blockchain.clients.bitcoin

import com.wallet.core.primitives.UTXO
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemTransactionLoadMetadata

class BitcoinChainDataTest {

    @Test
    fun `zcash metadata keeps branch id`() {
        val metadata = ZCashChainData(
            utxo = listOf(
                UTXO(
                    transaction_id = "transaction",
                    vout = 1,
                    value = "1000",
                    address = "address",
                )
            ),
            branchId = "c2d6d0b4",
        ).toDto()

        val zcash = metadata as GemTransactionLoadMetadata.Zcash
        assertEquals("c2d6d0b4", zcash.branchId)
        assertEquals("1000", zcash.utxos.first().value)
    }
}
