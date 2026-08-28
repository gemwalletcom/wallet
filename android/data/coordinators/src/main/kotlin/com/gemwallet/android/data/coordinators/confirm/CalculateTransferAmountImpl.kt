package com.gemwallet.android.data.coordinators.confirm

import com.gemwallet.android.application.confirm.cases.CalculateTransferAmount
import com.gemwallet.android.domains.confirm.BalanceRequirement
import com.gemwallet.android.domains.confirm.ConfirmError
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.ConfirmParams
import com.wallet.core.primitives.Asset
import uniffi.gemstone.GemTransferAmountException
import uniffi.gemstone.GemTransferAmountInput
import uniffi.gemstone.calculateTransferAmount
import java.math.BigInteger

class CalculateTransferAmountImpl : CalculateTransferAmount {

    override fun invoke(
        params: ConfirmParams,
        availableValue: BigInteger,
        feeAssetInfo: AssetInfo,
        fee: BigInteger,
    ): BigInteger {
        val input = transferAmountInput(params, availableValue, feeAssetInfo, fee)
        return try {
            BigInteger(calculateTransferAmount(input).value)
        } catch (error: GemTransferAmountException) {
            throw error.toConfirmError(params.asset, feeAssetInfo.asset)
        }
    }
}

internal fun transferAmountInput(
    params: ConfirmParams,
    availableValue: BigInteger,
    feeAssetInfo: AssetInfo,
    fee: BigInteger,
) = GemTransferAmountInput(
    inputType = params.toDto(),
    value = params.amount.toString(),
    availableValue = availableValue.toString(),
    feeAsset = feeAssetInfo.asset.id.toIdentifier(),
    feeAssetBalance = feeAssetInfo.balance.balance.available,
    fee = fee.toString(),
    isMaxAmount = params.useMaxAmount,
    minimumValue = params.minimumAmount?.toString(),
)

internal fun GemTransferAmountException.toConfirmError(asset: Asset, feeAsset: Asset): ConfirmError = when (this) {
    is GemTransferAmountException.InsufficientBalance -> ConfirmError.InsufficientBalance(
        asset = asset(assetId, asset, feeAsset),
        requirement = requirement(required, available),
    )
    is GemTransferAmountException.InsufficientNetworkFee -> ConfirmError.InsufficientFee(
        chain = feeAsset.id.chain,
        requirement = requirement(required, available),
    )
    is GemTransferAmountException.MinimumAccountBalanceTooLow -> ConfirmError.MinimumAccountBalanceTooLow(
        asset = asset(assetId, asset, feeAsset),
        requirement = requirement(required, available),
    )
}

private fun asset(assetId: String, asset: Asset, feeAsset: Asset): Asset =
    if (asset.id.toIdentifier() == assetId) asset else feeAsset

private fun requirement(required: String, available: String) = BalanceRequirement(
    required = BigInteger(required),
    available = BigInteger(available),
)
