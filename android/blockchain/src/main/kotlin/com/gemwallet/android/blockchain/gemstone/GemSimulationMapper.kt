package com.gemwallet.android.blockchain.gemstone

import com.gemwallet.android.ext.toAssetId
import com.wallet.core.primitives.SimulationBalanceChange
import com.wallet.core.primitives.SimulationHeader
import com.wallet.core.primitives.SimulationPayloadField
import com.wallet.core.primitives.SimulationPayloadFieldDisplay
import com.wallet.core.primitives.SimulationPayloadFieldKind
import com.wallet.core.primitives.SimulationPayloadFieldType
import com.wallet.core.primitives.SimulationResult
import com.wallet.core.primitives.SimulationSeverity
import com.wallet.core.primitives.SimulationWarning
import com.wallet.core.primitives.SimulationWarningApproval
import com.wallet.core.primitives.SimulationWarningType
import uniffi.gemstone.SimulationResult as GemSimulationResult

fun GemSimulationResult.toPrimitives(): SimulationResult = SimulationResult(
    warnings = warnings.mapNotNull { it.toPrimitives() },
    balanceChanges = balanceChanges.mapNotNull { it.toPrimitives() },
    payload = payload.map { it.toPrimitives() },
    header = header?.toPrimitives(),
)

private fun uniffi.gemstone.SimulationWarning.toPrimitives(): SimulationWarning? = warning.toPrimitives()?.let {
    SimulationWarning(severity = severity.toPrimitives(), warning = it, message = message)
}

private fun uniffi.gemstone.SimulationSeverity.toPrimitives(): SimulationSeverity = when (this) {
    uniffi.gemstone.SimulationSeverity.LOW -> SimulationSeverity.Low
    uniffi.gemstone.SimulationSeverity.WARNING -> SimulationSeverity.Warning
    uniffi.gemstone.SimulationSeverity.CRITICAL -> SimulationSeverity.Critical
}

private fun uniffi.gemstone.SimulationWarningType.toPrimitives(): SimulationWarningType? = when (this) {
    is uniffi.gemstone.SimulationWarningType.TokenApproval -> v1.assetId.toAssetId()?.let {
        SimulationWarningType.TokenApproval(SimulationWarningApproval(it, v1.value))
    }
    uniffi.gemstone.SimulationWarningType.SuspiciousSpender -> SimulationWarningType.SuspiciousSpender
    uniffi.gemstone.SimulationWarningType.ExternallyOwnedSpender -> SimulationWarningType.ExternallyOwnedSpender
    is uniffi.gemstone.SimulationWarningType.NftCollectionApproval -> v1.toAssetId()?.let {
        SimulationWarningType.NftCollectionApproval(it)
    }
    is uniffi.gemstone.SimulationWarningType.PermitApproval -> v1.assetId.toAssetId()?.let {
        SimulationWarningType.PermitApproval(SimulationWarningApproval(it, v1.value))
    }
    is uniffi.gemstone.SimulationWarningType.PermitBatchApproval -> SimulationWarningType.PermitBatchApproval(v1)
    uniffi.gemstone.SimulationWarningType.ValidationError -> SimulationWarningType.ValidationError
}

private fun uniffi.gemstone.SimulationBalanceChange.toPrimitives(): SimulationBalanceChange? {
    val id = assetId.toAssetId() ?: return null
    return SimulationBalanceChange(assetId = id, value = value)
}

private fun uniffi.gemstone.SimulationHeader.toPrimitives(): SimulationHeader? {
    val id = assetId.toAssetId() ?: return null
    return SimulationHeader(assetId = id, value = value, isUnlimited = isUnlimited)
}

fun uniffi.gemstone.SimulationPayloadField.toPrimitives(): SimulationPayloadField = SimulationPayloadField(
    kind = kind.toPrimitives(),
    label = label,
    value = value,
    fieldType = fieldType.toPrimitives(),
    display = display.toPrimitives(),
)

private fun uniffi.gemstone.SimulationPayloadFieldKind.toPrimitives(): SimulationPayloadFieldKind = when (this) {
    uniffi.gemstone.SimulationPayloadFieldKind.CONTRACT -> SimulationPayloadFieldKind.Contract
    uniffi.gemstone.SimulationPayloadFieldKind.METHOD -> SimulationPayloadFieldKind.Method
    uniffi.gemstone.SimulationPayloadFieldKind.TOKEN -> SimulationPayloadFieldKind.Token
    uniffi.gemstone.SimulationPayloadFieldKind.SPENDER -> SimulationPayloadFieldKind.Spender
    uniffi.gemstone.SimulationPayloadFieldKind.VALUE -> SimulationPayloadFieldKind.Value
    uniffi.gemstone.SimulationPayloadFieldKind.CUSTOM -> SimulationPayloadFieldKind.Custom
}

private fun uniffi.gemstone.SimulationPayloadFieldType.toPrimitives(): SimulationPayloadFieldType = when (this) {
    uniffi.gemstone.SimulationPayloadFieldType.TEXT -> SimulationPayloadFieldType.Text
    uniffi.gemstone.SimulationPayloadFieldType.ADDRESS -> SimulationPayloadFieldType.Address
    uniffi.gemstone.SimulationPayloadFieldType.TIMESTAMP -> SimulationPayloadFieldType.Timestamp
}

private fun uniffi.gemstone.SimulationPayloadFieldDisplay.toPrimitives(): SimulationPayloadFieldDisplay = when (this) {
    uniffi.gemstone.SimulationPayloadFieldDisplay.PRIMARY -> SimulationPayloadFieldDisplay.Primary
    uniffi.gemstone.SimulationPayloadFieldDisplay.SECONDARY -> SimulationPayloadFieldDisplay.Secondary
}

fun SimulationPayloadField.toGem(): uniffi.gemstone.SimulationPayloadField = uniffi.gemstone.SimulationPayloadField(
    kind = kind.toGem(),
    label = label,
    value = value,
    fieldType = fieldType.toGem(),
    display = display.toGem(),
)

private fun SimulationPayloadFieldKind.toGem(): uniffi.gemstone.SimulationPayloadFieldKind = when (this) {
    SimulationPayloadFieldKind.Contract -> uniffi.gemstone.SimulationPayloadFieldKind.CONTRACT
    SimulationPayloadFieldKind.Method -> uniffi.gemstone.SimulationPayloadFieldKind.METHOD
    SimulationPayloadFieldKind.Token -> uniffi.gemstone.SimulationPayloadFieldKind.TOKEN
    SimulationPayloadFieldKind.Spender -> uniffi.gemstone.SimulationPayloadFieldKind.SPENDER
    SimulationPayloadFieldKind.Value -> uniffi.gemstone.SimulationPayloadFieldKind.VALUE
    SimulationPayloadFieldKind.Custom -> uniffi.gemstone.SimulationPayloadFieldKind.CUSTOM
}

private fun SimulationPayloadFieldType.toGem(): uniffi.gemstone.SimulationPayloadFieldType = when (this) {
    SimulationPayloadFieldType.Text -> uniffi.gemstone.SimulationPayloadFieldType.TEXT
    SimulationPayloadFieldType.Address -> uniffi.gemstone.SimulationPayloadFieldType.ADDRESS
    SimulationPayloadFieldType.Timestamp -> uniffi.gemstone.SimulationPayloadFieldType.TIMESTAMP
}

private fun SimulationPayloadFieldDisplay.toGem(): uniffi.gemstone.SimulationPayloadFieldDisplay = when (this) {
    SimulationPayloadFieldDisplay.Primary -> uniffi.gemstone.SimulationPayloadFieldDisplay.PRIMARY
    SimulationPayloadFieldDisplay.Secondary -> uniffi.gemstone.SimulationPayloadFieldDisplay.SECONDARY
}
