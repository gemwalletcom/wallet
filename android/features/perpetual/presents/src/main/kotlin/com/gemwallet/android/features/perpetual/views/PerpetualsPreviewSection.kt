package com.gemwallet.android.features.perpetual.views

import androidx.compose.foundation.layout.Column
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.perpetual.viewmodels.PerpetualsPreviewViewModel
import com.gemwallet.android.features.perpetual.views.components.PerpetualPositionItem
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.clickable
import com.gemwallet.android.ui.components.list_item.LinkItem
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.models.ListPosition

@Composable
fun PerpetualsPreviewSection(
    onOpenPerpetuals: () -> Unit,
    onOpenPerpetualDetails: (String) -> Unit,
    viewModel: PerpetualsPreviewViewModel = hiltViewModel(),
) {
    val show by viewModel.showPerpetuals.collectAsStateWithLifecycle()
    if (!show) return
    val positions by viewModel.positions.collectAsStateWithLifecycle()

    Column {
        SubheaderItem(stringResource(R.string.perpetuals_title), onClick = onOpenPerpetuals)
        if (positions.isEmpty()) {
            LinkItem(
                title = stringResource(R.string.banner_perpetuals_title),
                icon = R.drawable.settings_pricealert,
                listPosition = ListPosition.Single,
                onClick = onOpenPerpetuals,
            )
        } else {
            positions.forEachIndexed { index, position ->
                PerpetualPositionItem(
                    data = position,
                    listPosition = ListPosition.getPosition(index, positions.size),
                    modifier = Modifier.clickable { onOpenPerpetualDetails(position.perpetualId) },
                )
            }
        }
    }
}
