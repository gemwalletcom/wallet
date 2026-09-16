package com.gemwallet.android.features.transfer_amount.presents

import androidx.compose.runtime.Composable
import uniffi.gemstone.GemStakeValidatorSelection
import com.gemwallet.android.ext.toPrimitives
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.screen.FatalStateScene
import com.gemwallet.android.ui.components.screen.LoadingScene

@Composable
fun ValidatorsScreen(
    selection: GemStakeValidatorSelection,
    selectedValidatorId: String,
    onCancel: () -> Unit,
    onSelect: (String) -> Unit
) {
    if (selection.options.isEmpty()) {
        FatalStateScene(
            title = stringResource(id = R.string.stake_validators),
            message = "Validators not found",
            onCancel = onCancel
        )
        return
    }
    ValidatorsScene(
        recommended = selection.recommended,
        validators = selection.options,
        selectedValidatorId = selectedValidatorId,
        onCancel = onCancel,
        onSelect = onSelect,
    )
}
