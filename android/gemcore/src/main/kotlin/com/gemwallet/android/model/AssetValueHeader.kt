package com.gemwallet.android.model

import com.gemwallet.android.ext.toPrimitives
import com.wallet.core.primitives.Asset
import uniffi.gemstone.GemApprovalValue
import uniffi.gemstone.GemSimulationValue
import java.math.BigInteger

data class AssetValueHeader(
    val asset: Asset,
    val value: ApprovalValue,
)

sealed interface ApprovalValue {
    data class Exact(val value: BigInteger) : ApprovalValue
    data object Unlimited : ApprovalValue
}

fun GemSimulationValue.toAssetValueHeader() = AssetValueHeader(
    asset = asset.toPrimitives(),
    value = when (val value = value) {
        is GemApprovalValue.Exact -> ApprovalValue.Exact(value.value)
        GemApprovalValue.Unlimited -> ApprovalValue.Unlimited
    },
)
