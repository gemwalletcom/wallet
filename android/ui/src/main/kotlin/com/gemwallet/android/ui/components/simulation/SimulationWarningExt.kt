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

fun SimulationWarning.isVisible(): Boolean = when (val warningType = warning) {
    is SimulationWarningType.TokenApproval -> warningType.content.value == null
    is SimulationWarningType.PermitApproval -> warningType.content.value == null
    is SimulationWarningType.PermitBatchApproval -> warningType.content == null
    SimulationWarningType.SuspiciousSpender,
    SimulationWarningType.ExternallyOwnedSpender,
    is SimulationWarningType.NftCollectionApproval,
    SimulationWarningType.ValidationError -> true
}

@StringRes
fun SimulationWarning.titleRes(): Int? = when (warning) {
    SimulationWarningType.ValidationError -> if (severity != SimulationSeverity.Critical) R.string.common_warning else R.string.errors_error_occured
    is SimulationWarningType.NftCollectionApproval -> R.string.simulation_warning_nft_collection_approval_title
    is SimulationWarningType.TokenApproval,
    is SimulationWarningType.PermitApproval,
    is SimulationWarningType.PermitBatchApproval -> if (isVisible()) R.string.simulation_warning_unlimited_token_approval_title else null
    SimulationWarningType.SuspiciousSpender,
    SimulationWarningType.ExternallyOwnedSpender -> R.string.errors_error_occured
}

@StringRes
fun SimulationWarning.descriptionRes(): Int? = when (warning) {
    is SimulationWarningType.TokenApproval,
    is SimulationWarningType.PermitApproval,
    is SimulationWarningType.PermitBatchApproval -> if (isVisible()) R.string.simulation_warning_unlimited_token_approval_description else null
    SimulationWarningType.SuspiciousSpender,
    SimulationWarningType.ExternallyOwnedSpender -> R.string.common_suspicious_address
    SimulationWarningType.ValidationError -> if (severity == SimulationSeverity.Critical) R.string.errors_error_occured else null
    is SimulationWarningType.NftCollectionApproval -> null
}

@Composable
fun SimulationWarning.descriptionText(): String? = when (warning) {
    SimulationWarningType.ValidationError -> if (severity != SimulationSeverity.Critical) message.orEmpty() else message ?: stringResource(R.string.errors_error_occured)
    else -> message ?: descriptionRes()?.let { stringResource(it) }
}
