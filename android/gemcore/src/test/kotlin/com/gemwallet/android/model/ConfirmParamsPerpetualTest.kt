package com.gemwallet.android.model

import com.gemwallet.android.serializer.jsonEncoder
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAssetHyperCoreUBTC
import com.gemwallet.android.testkit.mockPerpetualConfirmData
import com.gemwallet.android.testkit.mockPerpetualReduceData
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.PerpetualType
import com.wallet.core.primitives.TransactionType
import kotlinx.serialization.encodeToString
import org.junit.Assert.assertEquals
import org.junit.Test
import java.math.BigInteger

class ConfirmParamsPerpetualTest {

    private val asset = mockAssetHyperCoreUBTC()
    private val account = mockAccount(chain = Chain.HyperCore)
    private val amount = BigInteger("123456")

    private fun perpetualParams(perpetualType: PerpetualType): ConfirmParams.PerpetualParams =
        ConfirmParams.PerpetualParams(
            asset = asset,
            from = account,
            amount = amount,
            useMaxAmount = false,
            perpetualType = perpetualType,
        )

    @Test
    fun serialization_open_roundTripsPerpetualType() {
        val original: ConfirmParams = perpetualParams(
            PerpetualType.Open(mockPerpetualConfirmData(direction = PerpetualDirection.Long)),
        )
        val json = jsonEncoder.encodeToString(original)
        assertEquals(original, jsonEncoder.decodeFromString<ConfirmParams>(json))
    }

    @Test
    fun serialization_close_roundTripsPerpetualType() {
        val original: ConfirmParams = perpetualParams(
            PerpetualType.Close(mockPerpetualConfirmData(direction = PerpetualDirection.Short)),
        )
        val json = jsonEncoder.encodeToString(original)
        assertEquals(original, jsonEncoder.decodeFromString<ConfirmParams>(json))
    }

    @Test
    fun serialization_increase_roundTripsPerpetualType() {
        val original: ConfirmParams = perpetualParams(
            PerpetualType.Increase(mockPerpetualConfirmData()),
        )
        val json = jsonEncoder.encodeToString(original)
        assertEquals(original, jsonEncoder.decodeFromString<ConfirmParams>(json))
    }

    @Test
    fun serialization_reduce_roundTripsPerpetualType() {
        val original: ConfirmParams = perpetualParams(
            PerpetualType.Reduce(mockPerpetualReduceData(positionDirection = PerpetualDirection.Long)),
        )
        val json = jsonEncoder.encodeToString(original)
        assertEquals(original, jsonEncoder.decodeFromString<ConfirmParams>(json))
    }

    @Test
    fun destination_isHyperliquidProviderForAllPerpetualVariants() {
        val variants = listOf(
            PerpetualType.Open(mockPerpetualConfirmData()),
            PerpetualType.Close(mockPerpetualConfirmData()),
            PerpetualType.Increase(mockPerpetualConfirmData()),
            PerpetualType.Reduce(mockPerpetualReduceData()),
        )
        variants.forEach { perpetualType ->
            val destination = perpetualParams(perpetualType).destination()
            assertEquals("", destination.address)
            assertEquals("Hyperliquid", destination.name)
        }
    }

    @Test
    fun getTransactionType_mapsEachPerpetualVariantCorrectly() {
        assertEquals(
            TransactionType.PerpetualOpenPosition,
            perpetualParams(PerpetualType.Open(mockPerpetualConfirmData())).getTransactionType(),
        )
        assertEquals(
            TransactionType.PerpetualClosePosition,
            perpetualParams(PerpetualType.Close(mockPerpetualConfirmData())).getTransactionType(),
        )
        assertEquals(
            TransactionType.PerpetualModifyPosition,
            perpetualParams(PerpetualType.Increase(mockPerpetualConfirmData())).getTransactionType(),
        )
        assertEquals(
            TransactionType.PerpetualModifyPosition,
            perpetualParams(PerpetualType.Reduce(mockPerpetualReduceData())).getTransactionType(),
        )
    }
}
