package com.gemwallet.android.blockchain.services

import uniffi.gemstone.GemRecipient
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.domains.confirm.confirmInput
import com.gemwallet.android.domains.confirm.transfer
import uniffi.gemstone.GemTransactionInputType
import uniffi.gemstone.GemTransferData
import com.gemwallet.android.model.FeeAssetSelection
import com.gemwallet.android.model.FeeSelection
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAssetEthereum
import com.gemwallet.android.testkit.mockAssetTempoUSDCe
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.FeePriority
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemConfirmData
import uniffi.gemstone.GemConfirmFeeSelection
import uniffi.gemstone.GemFeeOptions
import uniffi.gemstone.GemFeeRate
import uniffi.gemstone.GemGasPriceType
import uniffi.gemstone.GemTransactionLoadFee
import uniffi.gemstone.GemTransactionLoadMetadata
import java.math.BigInteger
import com.gemwallet.android.ext.toIdentifier
import uniffi.gemstone.GemAssetBalance
import uniffi.gemstone.GemConfirmMetadata
import uniffi.gemstone.GemConfirmPreload
import uniffi.gemstone.GemTransferAmount
import uniffi.gemstone.GemTransferAmountResult

class GemConfirmMapperTest {

    @Test
    fun confirmLoadOptions_mapsTheFeeAndFeeAssetSelection() {
        val options = confirmLoadOptions(FeeSelection.Custom(BigInteger("42")), FeeAssetSelection.Selected(mockAssetTempoUSDCe().id))

        assertEquals(GemConfirmFeeSelection.Custom("42"), options.feeSelection)
        assertEquals("tempo_0x20C000000000000000000000b9537d11c60E8b50", options.feeAssetId)
        assertEquals(null, confirmLoadOptions(FeeSelection.Preset(FeePriority.Fast), FeeAssetSelection.Automatic).feeAssetId)
    }

    @Test
    fun toSignerParams_assemblesTheFeeFromTheSelectedPriority() {
        val asset = mockAssetEthereum()
        val input = GemTransferData(
            inputType = GemTransactionInputType.transfer(asset),
            recipient = GemRecipient("0xrecipient"),
            value = "1000000000000000",
        ).confirmInput(mockAccount(chain = Chain.Ethereum))
        val feeRates = listOf(
            GemFeeRate(FeePriority.Normal.toGem(), GemGasPriceType.Eip1559(gasPrice = "2", priorityFee = "3")),
            GemFeeRate(FeePriority.Fast.toGem(), GemGasPriceType.Eip1559(gasPrice = "1", priorityFee = "1")),
        )
        val preload = GemConfirmPreload(
            confirmData = GemConfirmData(
                fee = GemTransactionLoadFee(
                    fee = "21000",
                    gasPriceType = feeRates[0].gasPriceType,
                    gasLimit = "21000",
                    options = GemFeeOptions(emptyMap()),
                    feeAsset = asset.id.chain.string,
                ),
                selectedPriority = FeePriority.Normal.toGem(),
                feeRates = feeRates,
                metadata = GemTransactionLoadMetadata.None,
                simulation = null,
                input = input,
            ),
            metadata = GemConfirmMetadata(
                assetBalance = gemBalance(asset.id.toIdentifier()),
                feeAssetBalance = gemBalance(asset.id.toIdentifier()),
                prices = emptyList(),
            ),
            feeAsset = asset.toGem(),
            amount = GemTransferAmountResult.Amount(GemTransferAmount(value = "1", networkFee = "1", isMaxAmount = false)),
        )

        val result = preload.toSignerParams(input)

        assertEquals(FeePriority.Normal, result.fee.priority)
        assertEquals(BigInteger("21000"), result.fee.amount)
        assertEquals(feeRates, result.feeRates)
        assertEquals(GemTransactionLoadMetadata.None, result.confirmData.metadata)
    }
}

private fun gemBalance(assetId: String) = GemAssetBalance(
    assetId = assetId,
    available = "0",
    frozen = "0",
    locked = "0",
    staked = "0",
    pending = "0",
    pendingUnconfirmed = "0",
    rewards = "0",
    reserved = "0",
    withdrawable = "0",
    earn = "0",
    metadata = null,
)
