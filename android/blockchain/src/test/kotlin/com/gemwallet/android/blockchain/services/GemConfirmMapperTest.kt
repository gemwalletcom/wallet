package com.gemwallet.android.blockchain.services

import uniffi.gemstone.GemRecipient
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.domains.confirm.transfer
import uniffi.gemstone.GemTransactionInputType
import uniffi.gemstone.GemTransferData
import com.gemwallet.android.model.FeeAssetSelection
import com.gemwallet.android.model.FeeSelection
import com.gemwallet.android.testkit.mockGemConfirmMetadata
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAssetEthereum
import com.gemwallet.android.testkit.mockAssetTempoUSDCe
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.FeePriority
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemConfirmData
import uniffi.gemstone.GemConfirmInput
import uniffi.gemstone.GemConfirmFeeSelection
import uniffi.gemstone.GemFeeOptions
import uniffi.gemstone.GemFeeRate
import uniffi.gemstone.GemGasPriceType
import uniffi.gemstone.GemTransactionLoadFee
import uniffi.gemstone.GemTransactionLoadMetadata
import java.math.BigInteger
import uniffi.gemstone.GemConfirmPreload
import uniffi.gemstone.GemTransferAmount
import uniffi.gemstone.GemTransferAmountResult

class GemConfirmMapperTest {

    @Test
    fun confirmLoadOptions_mapsTheFeeAndFeeAssetSelection() {
        val options = confirmLoadOptions(FeeSelection.Custom(BigInteger("42")), FeeAssetSelection.Selected(mockAssetTempoUSDCe().id))

        assertEquals(GemConfirmFeeSelection.Custom(BigInteger("42")), options.feeSelection)
        assertEquals("tempo_0x20C000000000000000000000b9537d11c60E8b50", options.feeAssetId)
        assertEquals(null, confirmLoadOptions(FeeSelection.Preset(FeePriority.Fast), FeeAssetSelection.Automatic).feeAssetId)
    }

    @Test
    fun toSignerParams_assemblesTheFeeFromTheSelectedPriority() {
        val asset = mockAssetEthereum()
        val input = GemConfirmInput(
            from = mockAccount(chain = Chain.Ethereum).toGem(),
            transfer = GemTransferData(
                inputType = GemTransactionInputType.transfer(asset),
                recipient = GemRecipient("0xrecipient"),
                value = BigInteger("1000000000000000"),
            ),
        )
        val feeRates = listOf(
            GemFeeRate(FeePriority.Normal.toGem(), GemGasPriceType.Eip1559(gasPrice = BigInteger("2"), priorityFee = BigInteger("3"))),
            GemFeeRate(FeePriority.Fast.toGem(), GemGasPriceType.Eip1559(gasPrice = BigInteger.ONE, priorityFee = BigInteger.ONE)),
        )
        val preload = GemConfirmPreload(
            confirmData = GemConfirmData(
                fee = GemTransactionLoadFee(
                    fee = BigInteger("21000"),
                    gasPriceType = feeRates[0].gasPriceType,
                    gasLimit = BigInteger("21000"),
                    options = GemFeeOptions(emptyMap()),
                    feeAsset = asset.id.chain.string,
                ),
                selectedPriority = FeePriority.Normal.toGem(),
                feeRates = feeRates,
                metadata = GemTransactionLoadMetadata.None,
                simulation = null,
                input = input,
            ),
            amount = GemTransferAmountResult.Amount(GemTransferAmount(value = BigInteger.ONE, networkFee = BigInteger.ONE, isMaxAmount = false)),
        )

        val result = preload.toSignerParams()

        assertEquals(FeePriority.Normal, result.fee.priority)
        assertEquals(BigInteger("21000"), result.fee.amount)
        assertEquals(GemTransactionLoadMetadata.None, result.confirmData.metadata)
    }
}
