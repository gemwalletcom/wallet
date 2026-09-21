package com.gemwallet.android.features.transfer_amount.presents

import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import com.gemwallet.android.features.transfer_amount.viewmodels.models.ValidatorsUIModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.empty.EmptyContentType
import com.gemwallet.android.ui.components.empty.EmptyContentView
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.ValidatorItem
import com.gemwallet.android.ui.components.list_item.ValidatorRowUIModel
import com.gemwallet.android.ui.components.list_item.aprText
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.theme.WalletTheme

@Composable
fun ValidatorsScene(selection: ValidatorsUIModel, selectedValidatorId: String, onSelect: (String) -> Unit, onCancel: () -> Unit) {
    Scene(
        title = stringResource(id = R.string.stake_validators),
        onClose = onCancel,
    ) {
        LazyColumn {
            if (selection.recommended.isEmpty() && selection.options.isEmpty()) {
                item {
                    EmptyContentView(type = EmptyContentType.Validators, modifier = Modifier.fillParentMaxSize())
                }
            }
            if (selection.recommended.isNotEmpty()) {
                item {
                    SubheaderItem(R.string.common_recommended)
                }
                itemsPositioned(selection.recommended, key = { _, item -> "recommended-${item.id}" }) { position, item ->
                    ValidatorItem(
                        data = item,
                        listPosition = position,
                        isSelected = selectedValidatorId == item.id,
                        onClick = onSelect,
                    )
                }
            }
            if (selection.options.isNotEmpty()) {
                item {
                    SubheaderItem(R.string.stake_active)
                }
                itemsPositioned(selection.options, key = { _, item -> item.id }) { position, item ->
                    ValidatorItem(
                        data = item,
                        listPosition = position,
                        isSelected = selectedValidatorId == item.id,
                        onClick = onSelect,
                    )
                }
            }
        }
    }
}

@Composable
@Preview
fun PreviewValidatorsScene() {
    WalletTheme {
        ValidatorsScene(
            selection = ValidatorsUIModel(
                recommended = emptyList(),
                options = listOf(
                    previewRow("some_validator_id", "Castlenode", 9.10),
                    previewRow("some_validator_id_1", "Ubik Capital 0%Fee", 10.000),
                    previewRow("some_validator_id_2", "Virtual Hive", 9.50),
                ),
            ),
            selectedValidatorId = "some_validator_id_1",
            onCancel = {},
            onSelect = {},
        )
    }
}

private fun previewRow(id: String, name: String, apr: Double) = ValidatorRowUIModel(
    id = id,
    name = name,
    imageUrl = null,
    placeholder = name.take(1),
    apr = apr.aprText(),
)
