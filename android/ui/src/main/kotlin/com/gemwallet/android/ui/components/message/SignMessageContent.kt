@file:OptIn(ExperimentalMaterial3Api::class)

package com.gemwallet.android.ui.components.message

import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Text
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.SubheaderItem
import com.gemwallet.android.ui.components.list_item.listItem
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.components.screen.ModalBottomSheet
import com.gemwallet.android.ui.components.simulation.simulationPayloadDetailsContent
import com.gemwallet.android.ui.models.ListPosition
import com.gemwallet.android.ui.models.PayloadField
import com.gemwallet.android.ui.theme.paddingDefault

enum class SignMessageSheetType {
    Details,
    FullMessage,
}

fun LazyListScope.signMessageText(message: String) {
    item {
        SubheaderItem(R.string.sign_message_message)
        Text(
            modifier = Modifier
                .fillMaxWidth()
                .listItem()
                .padding(paddingDefault),
            text = message,
        )
    }
}

@Composable
fun SignMessagePayloadDetailsSheet(
    primaryFields: List<PayloadField>,
    secondaryFields: List<PayloadField>,
    onViewFullMessage: () -> Unit,
    onDismissRequest: () -> Unit,
) {
    ModalBottomSheet(
        onDismissRequest = onDismissRequest,
        sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true),
        title = stringResource(R.string.common_details),
    ) {
        LazyColumn {
            simulationPayloadDetailsContent(
                primaryFields = primaryFields,
                secondaryFields = secondaryFields,
            )
            item {
                PropertyItem(
                    action = R.string.sign_message_view_full_message,
                    listPosition = ListPosition.Single,
                    onClick = onViewFullMessage,
                )
            }
        }
    }
}

@Composable
fun SignMessageFullMessageSheet(
    message: String,
    onDismissRequest: () -> Unit,
) {
    ModalBottomSheet(
        onDismissRequest = onDismissRequest,
        sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true),
        title = stringResource(R.string.sign_message_view_full_message),
    ) {
        LazyColumn(
            contentPadding = PaddingValues(paddingDefault),
        ) {
            item {
                Text(
                    modifier = Modifier.fillMaxWidth(),
                    text = message,
                )
            }
        }
    }
}
