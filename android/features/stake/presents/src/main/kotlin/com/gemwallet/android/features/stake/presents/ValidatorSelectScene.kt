package com.gemwallet.android.features.stake.presents

import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.gemGroupList
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.theme.WalletTheme
import uniffi.gemstone.DelegationValidator
import uniffi.gemstone.GemGroup
import uniffi.gemstone.GemGroupList
import uniffi.gemstone.GemListItem
import uniffi.gemstone.GemListPhase
import uniffi.gemstone.GemListSectionFooter
import uniffi.gemstone.GemListSectionTitle
import uniffi.gemstone.GemLocalizedText
import uniffi.gemstone.GemValidatorRow
import uniffi.gemstone.StakeProviderType

@Composable
fun ValidatorSelectScene(selection: GemGroupList, onSelect: (String) -> Unit, onCancel: () -> Unit) {
    Scene(
        title = stringResource(id = R.string.stake_validators),
        onClose = onCancel,
    ) {
        LazyColumn {
            gemGroupList(selection) { item ->
                if (item is GemListItem.Validator) onSelect(item.row.validator.id)
            }
        }
    }
}

@Composable
@Preview
fun PreviewValidatorSelectScene() {
    WalletTheme {
        ValidatorSelectScene(
            selection = GemGroupList(
                groups = listOf(
                    GemGroup(
                        title = GemListSectionTitle.ACTIVE,
                        footer = GemListSectionFooter.NONE,
                        items = listOf(
                            GemListItem.Validator(previewRow("some_validator_id", "Castlenode"), false),
                            GemListItem.Validator(previewRow("some_validator_id_1", "Ubik Capital 0%Fee"), true),
                            GemListItem.Validator(previewRow("some_validator_id_2", "Virtual Hive"), false),
                        ),
                    ),
                ),
                phase = GemListPhase.Rows,
            ),
            onCancel = {},
            onSelect = {},
        )
    }
}

private fun previewRow(id: String, name: String) = GemValidatorRow(
    validator = DelegationValidator(
        chain = "cosmos",
        id = id,
        name = name,
        isActive = true,
        commission = 0.0,
        apr = 9.1,
        providerType = StakeProviderType.STAKE,
    ),
    name = name,
    imageUrl = "",
    placeholder = name.take(1),
    provider = null,
    apr = GemLocalizedText.Apr(null),
    explorer = null,
)
