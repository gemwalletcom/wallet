package com.gemwallet.android.blockchain.clients.solana

import com.gemwallet.android.model.Fee
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.FeePriority
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Test
import wallet.core.jni.proto.Solana
import java.math.BigInteger

class SolanaSignClientTest {

    @Test
    fun applyPriorityFee_setsComputeUnitPriceFromUnitFee() {
        val fee = solanaFee(unitFee = 25_000, minerFee = 2_500, limit = 100_000)
        val builder = Solana.SigningInput.newBuilder()

        SolanaSignClient.applyPriorityFee(builder, fee)

        assertEquals(100_000, builder.priorityFeeLimit.limit)
        assertEquals(25_000L, builder.priorityFeePrice.price)
    }

    @Test
    fun applyPriorityFee_skipsPriceWhenUnitFeeZero() {
        val fee = solanaFee(unitFee = 0, minerFee = 2_500, limit = 100_000)
        val builder = Solana.SigningInput.newBuilder()

        SolanaSignClient.applyPriorityFee(builder, fee)

        assertEquals(100_000, builder.priorityFeeLimit.limit)
        assertFalse(builder.hasPriorityFeePrice())
    }

    private fun solanaFee(unitFee: Long, minerFee: Long, limit: Long) = Fee.Solana(
        feeAssetId = AssetId(Chain.Solana),
        priority = FeePriority.Normal,
        amount = BigInteger.valueOf(5_000 + minerFee),
        minerFee = BigInteger.valueOf(minerFee),
        maxGasPrice = BigInteger.valueOf(5_000),
        unitFee = BigInteger.valueOf(unitFee),
        limit = BigInteger.valueOf(limit),
        options = emptyMap(),
    )
}
