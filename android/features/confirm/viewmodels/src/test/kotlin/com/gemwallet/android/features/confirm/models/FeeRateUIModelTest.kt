package com.gemwallet.android.domains.confirm

import com.gemwallet.android.testkit.mockAssetEthereum
import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.testkit.mockAssetPriceInfo
import com.gemwallet.android.testkit.mockAssetSolana
import com.wallet.core.primitives.FeePriority
import com.wallet.core.primitives.FeeUnitType
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemFeeRate
import uniffi.gemstone.GemGasPriceType
import java.math.BigInteger

class FeeRateUIModelTest {

    @Test
    fun feeRateScalesFiatFromSelectedLoadedFeeForGweiChain() {
        val assetInfo = mockAssetInfo(asset = mockAssetEthereum())
            .copy(price = mockAssetPriceInfo(price = 1.0))
        val selectedRate = GemFeeRate(
            priority = FeePriority.Normal.string,
            gasPriceType = GemGasPriceType.Eip1559(gasPrice = "2", priorityFee = "0"),
        )
        val model = FeeRateUIModel(
            feeRate = GemFeeRate(
                priority = FeePriority.Fast.string,
                gasPriceType = GemGasPriceType.Eip1559(gasPrice = "1", priorityFee = "0"),
            ),
            feeAsset = assetInfo,
            feeUnitType = FeeUnitType.Gwei,
            selectedRate = selectedRate,
            selectedFeeAmount = BigInteger("1000000000000000000"),
        )

        assertEquals(FeePriority.Fast, model.priority)
        assertEquals("$0.5", model.fiatValue)
    }

    @Test
    fun nativeFeeChainScalesCryptoFromSelectedLoadedFee() {
        val assetInfo = mockAssetInfo(asset = mockAssetSolana())
        val selectedRate = GemFeeRate(
            priority = FeePriority.Normal.string,
            gasPriceType = GemGasPriceType.Regular(gasPrice = "110"),
        )
        fun model(priority: FeePriority, gasPrice: String) = FeeRateUIModel(
            feeRate = GemFeeRate(priority = priority.string, gasPriceType = GemGasPriceType.Regular(gasPrice = gasPrice)),
            feeAsset = assetInfo,
            feeUnitType = FeeUnitType.Native,
            selectedRate = selectedRate,
            selectedFeeAmount = BigInteger("110000"),
        )

        assertEquals("0.0001 SOL", model(FeePriority.Slow, "100").price)
        assertEquals("0.00011 SOL", model(FeePriority.Normal, "110").price)
        assertEquals("0.0002 SOL", model(FeePriority.Fast, "200").price)
    }

    @Test
    fun nativeFeeChainShowsCryptoAmountWithoutFiatWhenFeeNotLoaded() {
        val model = FeeRateUIModel(
            feeRate = GemFeeRate(
                priority = FeePriority.Normal.string,
                gasPriceType = GemGasPriceType.Regular(gasPrice = "1"),
            ),
            feeAsset = mockAssetInfo(asset = mockAssetEthereum()),
            feeUnitType = FeeUnitType.Native,
        )

        assertEquals("0.000000000000000001 ETH", model.price)
        assertEquals("", model.fiatValue)
    }
}
