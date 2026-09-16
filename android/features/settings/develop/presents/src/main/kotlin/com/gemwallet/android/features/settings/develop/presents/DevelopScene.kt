package com.gemwallet.android.features.settings.develop.presents

import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.clipboard.setPlainText
import com.gemwallet.android.ui.components.list_item.LinkItem
import com.gemwallet.android.ui.components.list_item.property.PropertyItem
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.features.settings.develop.viewmodels.DevelopViewModel
import com.gemwallet.android.ui.components.clipboard.clipboardManager
import com.gemwallet.android.ui.theme.Placeholder

@Composable
fun DevelopScene(
    onInAppNotifications: () -> Unit,
    onPayments: () -> Unit,
    onCancel: () -> Unit,
    viewModel: DevelopViewModel = hiltViewModel(),
) {
    val context = LocalContext.current
    val clipboardManager = LocalContext.current.clipboardManager()
    val deviceId by viewModel.deviceId.collectAsState()
    val notificationsAvailable = viewModel.notificationsAvailable
    Scene(
        title = stringResource(id = R.string.settings_developer),
        onClose = onCancel,
    ) {
        LazyColumn {
            item {
                LinkItem(
                    title = "Payments",
                    onClick = onPayments,
                )
            }
            if (notificationsAvailable) {
                item {
                    LinkItem(
                        title = "In-App Notifications",
                        onClick = onInAppNotifications,
                    )
                }
            }
            item {
                PropertyItem("Clear Transactions", data = "") { viewModel.clearTransactions() }
                PropertyItem("Clear Pending Transactions", data = "") { viewModel.clearPendingTransactions() }
                PropertyItem("Clear Assets", data = "") { viewModel.clearAssets() }
                PropertyItem("Clear Delegations", data = "") { viewModel.clearDelegations() }
                PropertyItem("Clear Validators", data = "") { viewModel.clearValidators() }
                PropertyItem("Clear Banners", data = "") { viewModel.clearBanners() }
                PropertyItem("Activate All Cancelled Banners", data = "") { viewModel.activateCancelledBanners() }
                PropertyItem("Clear Prices", data = "") { viewModel.clearPrices() }
                PropertyItem("Clear Perpetuals", data = "") { viewModel.clearPerpetuals() }
            }
            item {
                PropertyItem("Device Id", data = deviceId.ifEmpty { Placeholder.empty }) {
                    clipboardManager.setPlainText(context, deviceId)
                }
                if (notificationsAvailable) {
                    val pushToken by viewModel.pushToken.collectAsState()
                    PropertyItem("Push token", data = pushToken.ifEmpty { Placeholder.empty }) {
                        clipboardManager.setPlainText(context, pushToken)
                    }
                }
                val platformStore by viewModel.platformStore.collectAsState()
                PropertyItem("Store", data = platformStore?.string ?: Placeholder.empty) {
                    clipboardManager.setPlainText(context, platformStore?.string ?: "")
                }
            }
        }
    }
}
