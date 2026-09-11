package com.gemwallet.android.ui.components.simulation

import androidx.annotation.StringRes
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.theme.pendingColor
import uniffi.gemstone.GemSimulationWarningKind
import uniffi.gemstone.GemSimulationWarningRow
import uniffi.gemstone.SimulationSeverity

@Composable
fun SimulationSeverity.color(): Color = when (this) {
    SimulationSeverity.CRITICAL -> MaterialTheme.colorScheme.error
    else -> pendingColor
}

@StringRes
fun GemSimulationWarningRow.titleRes(): Int = when (kind) {
    GemSimulationWarningKind.VALIDATION_ERROR -> if (severity != SimulationSeverity.CRITICAL) R.string.common_warning else R.string.errors_error_occurred
    GemSimulationWarningKind.NFT_COLLECTION_APPROVAL -> R.string.simulation_warning_nft_collection_approval_title
    GemSimulationWarningKind.UNLIMITED_APPROVAL -> R.string.simulation_warning_unlimited_token_approval_title
    GemSimulationWarningKind.EXTERNALLY_OWNED_SPENDER -> R.string.common_warning
    GemSimulationWarningKind.SUSPICIOUS_SPENDER -> R.string.errors_error_occurred
}

@StringRes
fun GemSimulationWarningRow.descriptionRes(): Int? = when (kind) {
    GemSimulationWarningKind.UNLIMITED_APPROVAL -> R.string.simulation_warning_unlimited_token_approval_description
    GemSimulationWarningKind.EXTERNALLY_OWNED_SPENDER -> R.string.simulation_warning_externally_owned_spender_description
    GemSimulationWarningKind.SUSPICIOUS_SPENDER -> R.string.common_suspicious_address
    GemSimulationWarningKind.VALIDATION_ERROR -> if (severity == SimulationSeverity.CRITICAL) R.string.errors_error_occurred else null
    GemSimulationWarningKind.NFT_COLLECTION_APPROVAL -> null
}

@Composable
fun GemSimulationWarningRow.descriptionText(): String? = when (kind) {
    GemSimulationWarningKind.VALIDATION_ERROR -> if (severity != SimulationSeverity.CRITICAL) message.orEmpty() else message ?: stringResource(R.string.errors_error_occurred)
    else -> message ?: descriptionRes()?.let { stringResource(it) }
}
