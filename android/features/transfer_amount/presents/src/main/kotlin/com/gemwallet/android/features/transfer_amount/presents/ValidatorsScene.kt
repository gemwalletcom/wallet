package com.gemwallet.android.features.transfer_amount.presents

import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.ValidatorItem
import com.gemwallet.android.ui.components.list_item.ValidatorRowUIModel
import com.gemwallet.android.ui.components.list_item.formatApr
import com.gemwallet.android.ui.components.list_item.property.itemsPositioned
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.theme.WalletTheme

@Composable
fun ValidatorsScene(
    recommended: List<ValidatorRowUIModel>,
    validators: List<ValidatorRowUIModel>,
    selectedValidatorId: String,
    onSelect: (String) -> Unit,
    onCancel: () -> Unit,
) {
    Scene(
        title = stringResource(id = R.string.stake_validators),
        onClose = onCancel,
    ) {
        LazyColumn {
            if (recommended.isNotEmpty()) {
                item {
                    SubheaderItem(R.string.common_recommended)
                }
                itemsPositioned(recommended, key = { index, item -> "recommended-${item.id}" }) { position, item ->
                    ValidatorItem(
                        data = item,
                        listPosition = position,
                        isSelected = selectedValidatorId == item.id,
                        onClick = onSelect
                    )
                }

            }
            item {
                SubheaderItem(R.string.stake_active)
            }
            val validatorsSize = validators.size
            itemsIndexed(validators, key = { index, item -> item.id }) { index, item ->
                ValidatorItem(
                    data = item,
                    listPosition = ListPosition.getPosition(index, validatorsSize),
                    isSelected = selectedValidatorId == item.id,
                    onClick = onSelect
                )
            }
        }
    }
}

@Composable
@Preview
fun PreviewValidatorsScene() {
    WalletTheme {
        ValidatorsScene(
            recommended = emptyList(),
            validators = listOf(
                previewRow("some_validator_id", "Castlenode", 9.10),
                previewRow("some_validator_id_1", "Ubik Capital 0%Fee", 10.000),
                previewRow("some_validator_id_2", "Virtual Hive", 9.50),
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
    aprText = apr.formatApr(),
)
