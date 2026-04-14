package com.gemwallet.android.domains.confirm

import com.gemwallet.android.testkit.mockAssetEthereum
import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.testkit.mockAssetPriceInfo
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
    fun nativeFeeChainShowsFiatAsPrimaryAndEmptyFiat() {
        val assetInfo = mockAssetInfo(asset = mockAssetEthereum())
            .copy(price = mockAssetPriceInfo(price = 1.0))
        val selectedRate = GemFeeRate(
            priority = FeePriority.Normal.string,
            gasPriceType = GemGasPriceType.Regular(gasPrice = "1"),
        )
        val model = FeeRateUIModel(
            feeRate = GemFeeRate(
                priority = FeePriority.Normal.string,
                gasPriceType = GemGasPriceType.Regular(gasPrice = "1"),
            ),
            feeAsset = assetInfo,
            feeUnitType = FeeUnitType.Native,
            selectedRate = selectedRate,
            selectedFeeAmount = BigInteger("1000000000000000000"),
        )

        assertEquals("$1.00", model.price)
        assertEquals("", model.fiatValue)
    }

    @Test
    fun feeRateReturnsEmptyWithoutLoadedFee() {
        val model = FeeRateUIModel(
            feeRate = GemFeeRate(
                priority = FeePriority.Normal.string,
                gasPriceType = GemGasPriceType.Regular(gasPrice = "1"),
            ),
            feeAsset = mockAssetInfo(asset = mockAssetEthereum()),
            feeUnitType = FeeUnitType.Native,
        )

        assertEquals("", model.price)
        assertEquals("", model.fiatValue)
    }
}
