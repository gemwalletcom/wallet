package com.gemwallet.android.blockchain.clients.solana

import com.gemwallet.android.testkit.mockFeeSolana
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Test
import wallet.core.jni.proto.Solana
import java.math.BigInteger

class SolanaSignClientTest {

    @Test
    fun applyPriorityFee_setsComputeUnitPriceFromUnitFee() {
        val fee = mockFeeSolana(
            unitFee = BigInteger.valueOf(25_000),
            minerFee = BigInteger.valueOf(2_500),
            limit = BigInteger.valueOf(100_000),
        )
        val builder = Solana.SigningInput.newBuilder()

        SolanaSignClient.applyPriorityFee(builder, fee)

        assertEquals(100_000, builder.priorityFeeLimit.limit)
        assertEquals(25_000L, builder.priorityFeePrice.price)
    }

    @Test
    fun applyPriorityFee_skipsPriceWhenUnitFeeZero() {
        val fee = mockFeeSolana(unitFee = BigInteger.ZERO)
        val builder = Solana.SigningInput.newBuilder()

        SolanaSignClient.applyPriorityFee(builder, fee)

        assertEquals(100_000, builder.priorityFeeLimit.limit)
        assertFalse(builder.hasPriorityFeePrice())
    }
}
