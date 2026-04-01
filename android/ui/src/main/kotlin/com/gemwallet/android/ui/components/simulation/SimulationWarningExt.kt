package com.gemwallet.android.ui.components.simulation

import androidx.annotation.StringRes
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.theme.pendingColor
import com.wallet.core.primitives.SimulationSeverity
import com.wallet.core.primitives.SimulationWarning
import com.wallet.core.primitives.SimulationWarningType

@Composable
fun SimulationSeverity.color(): Color = when (this) {
    SimulationSeverity.Critical -> MaterialTheme.colorScheme.error
    else -> pendingColor
}

@StringRes
fun SimulationWarning.titleRes(): Int = when (warning) {
    SimulationWarningType.ValidationError -> if (severity != SimulationSeverity.Critical) R.string.common_warning else R.string.errors_error_occured
    is SimulationWarningType.NftCollectionApproval -> R.string.simulation_warning_nft_collection_approval_title
    is SimulationWarningType.TokenApproval,
    is SimulationWarningType.PermitApproval,
    is SimulationWarningType.PermitBatchApproval -> R.string.simulation_warning_unlimited_token_approval_title
    else -> R.string.errors_error_occured
}

@Composable
fun SimulationWarning.descriptionText(): String? = when (warning) {
    SimulationWarningType.ValidationError -> if (severity != SimulationSeverity.Critical) message.orEmpty() else message ?: stringResource(R.string.errors_error_occured)
    is SimulationWarningType.TokenApproval,
    is SimulationWarningType.PermitApproval,
    is SimulationWarningType.PermitBatchApproval -> stringResource(R.string.simulation_warning_unlimited_token_approval_description)
    SimulationWarningType.SuspiciousSpender,
    SimulationWarningType.ExternallyOwnedSpender -> stringResource(R.string.common_suspicious_address)
    else -> message
}
