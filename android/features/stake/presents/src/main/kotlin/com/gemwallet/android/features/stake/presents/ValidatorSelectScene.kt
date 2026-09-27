package com.gemwallet.android.features.stake.presents

import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.empty.EmptyContentView
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.ValidatorItem
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.localization.titleRes
import com.gemwallet.android.ui.theme.WalletTheme
import uniffi.gemstone.DelegationValidator
import uniffi.gemstone.GemEmptyStateKind
import uniffi.gemstone.GemLocalizedText
import uniffi.gemstone.GemStakeValidatorOptions
import uniffi.gemstone.GemValidatorRow
import uniffi.gemstone.GemValidatorSection
import uniffi.gemstone.GemValidatorSectionKind
import uniffi.gemstone.StakeProviderType

@Composable
fun ValidatorSelectScene(selection: GemStakeValidatorOptions, selectedValidatorId: String, onSelect: (String) -> Unit, onCancel: () -> Unit) {
    Scene(
        title = stringResource(id = R.string.stake_validators),
        onClose = onCancel,
    ) {
        LazyColumn {
            if (selection.sections.isEmpty()) {
                item {
                    EmptyContentView(kind = GemEmptyStateKind.VALIDATORS, modifier = Modifier.fillParentMaxSize())
                }
            }
            selection.sections.forEach { section ->
                item {
                    SubheaderItem(section.kind.titleRes())
                }
                itemsPositioned(section.rows, key = { _, item -> "${section.kind}-${item.validator.id}" }) { position, item ->
                    ValidatorItem(
                        data = item,
                        listPosition = position,
                        isSelected = selectedValidatorId == item.validator.id,
                        onClick = onSelect,
                    )
                }
            }
        }
    }
}

@Composable
@Preview
fun PreviewValidatorSelectScene() {
    WalletTheme {
        ValidatorSelectScene(
            selection = GemStakeValidatorOptions(
                sections = listOf(
                    GemValidatorSection(
                        kind = GemValidatorSectionKind.ACTIVE,
                        rows = listOf(
                            previewRow("some_validator_id", "Castlenode"),
                            previewRow("some_validator_id_1", "Ubik Capital 0%Fee"),
                            previewRow("some_validator_id_2", "Virtual Hive"),
                        ),
                    ),
                ),
            ),
            selectedValidatorId = "some_validator_id_1",
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
