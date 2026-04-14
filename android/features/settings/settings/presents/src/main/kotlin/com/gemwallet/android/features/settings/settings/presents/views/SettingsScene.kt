@file:OptIn(ExperimentalMaterial3Api::class)

package com.gemwallet.android.features.settings.settings.presents.views

import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.ScrollState
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.settings.settings.viewmodels.SettingsViewModel
import com.gemwallet.android.ui.BuildConfig
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.PushRequest
import com.gemwallet.android.ui.components.list_item.LinkItem
import com.gemwallet.android.ui.components.list_item.property.DataBadgeChevron
import com.gemwallet.android.ui.components.list_item.property.PropertyDataText
import com.gemwallet.android.ui.components.screen.Scene
import com.gemwallet.android.ui.models.ListPosition

@OptIn(ExperimentalFoundationApi::class)
@Composable
fun SettingsScene(
    onSecurity: () -> Unit,
    onBridges: () -> Unit,
    onDevelop: () -> Unit,
    onWallets: () -> Unit,
    onAboutUs: () -> Unit,
    onNotifications: () -> Unit,
    @Suppress("UNUSED_PARAMETER") onPriceAlerts: () -> Unit,
    onSupport: () -> Unit,
    onPreferences: () -> Unit,
    onPerpetual: () -> Unit,
    onReferral: () -> Unit,
    scrollState: ScrollState = rememberScrollState()
) {
    val viewModel: SettingsViewModel = hiltViewModel()
    val uiState by viewModel.uiState.collectAsStateWithLifecycle()
    val isRewardsAvailable by viewModel.isRewardsAvailable.collectAsStateWithLifecycle()
    val walletsCount by viewModel.walletsCount.collectAsStateWithLifecycle()
    val pushEnabled by viewModel.pushEnabled.collectAsStateWithLifecycle()
    val supportState = rememberModalBottomSheetState(skipPartiallyExpanded = true)
    var isShowDevelopEnable by remember { mutableStateOf(false) }

    var requestPushGrant by remember { mutableStateOf<(() -> Unit)?>(null) }
    val notificationsAvailable = viewModel.isNotificationsAvailable()
    Scene(
        title = stringResource(id = R.string.settings_title),
        mainActionPadding = PaddingValues(0.dp),
        navigationBarPadding = false,
    ) {
        Column(
            modifier = Modifier
                .fillMaxSize()
                .verticalScroll(scrollState)
        ) {
            LinkItem(
                title = stringResource(id = R.string.wallets_title),
                icon = R.drawable.settings_wallets,
                listPosition = ListPosition.First,
                trailingContent = {
                    PropertyDataText(
                        text = walletsCount.toString(),
                        badge = { DataBadgeChevron() },
                    )
                },
                onClick = onWallets
            )
            LinkItem(
                title = stringResource(id = R.string.settings_security),
                icon = R.drawable.settings_security,
                listPosition = ListPosition.Last,
                onClick = onSecurity
            )
            if (notificationsAvailable) {
                LinkItem(
                    title = stringResource(id = R.string.settings_notifications_title),
                    icon = R.drawable.settings_notifications,
                    listPosition = ListPosition.First,
                    onClick = onNotifications,
                )
                LinkItem(
                    title = stringResource(id = R.string.settings_preferences_title),
                    icon = R.drawable.settings_preferences,
                    listPosition = ListPosition.Last,
                    onClick = onPreferences,
                )
            } else {
                LinkItem(
                    title = stringResource(id = R.string.settings_preferences_title),
                    icon = R.drawable.settings_preferences,
                    listPosition = ListPosition.Single,
                    onClick = onPreferences,
                )
            }
            LinkItem(
                title = stringResource(id = R.string.wallet_connect_title),
                icon = R.drawable.settings_wc,
                listPosition = ListPosition.Single,
            ) {
                onBridges()
            }

            LinkItem(
                title = stringResource(id = R.string.settings_support),
                icon = R.drawable.settings_support,
                listPosition = ListPosition.First,
            ) {
                if (!pushEnabled) {
                    requestPushGrant = {
                        viewModel.enableNotifications()
                        onSupport()
                    }
                } else {
                    onSupport()
                }
            }
            if (isRewardsAvailable) {
                LinkItem(
                    title = stringResource(id = R.string.rewards_title),
                    icon = R.drawable.settings_wallets,
                    listPosition = ListPosition.Middle
                ) {
                    onReferral()
                }
            }
            Box(modifier = Modifier.fillMaxWidth()) {
                LinkItem(
                    title = stringResource(id = R.string.settings_aboutus),
                    icon = R.drawable.settings_about_us,
                    listPosition = if (uiState.developEnabled) ListPosition.Middle else ListPosition.Last,
                    onClick = onAboutUs,
                    onLongClick = { isShowDevelopEnable = true }
                )
                DropdownMenu(
                    isShowDevelopEnable, { isShowDevelopEnable = false },
                    containerColor = MaterialTheme.colorScheme.background,
                ) {
                    DropdownMenuItem(
                        text = { Text("Enable develop") },
                        onClick = {
                            isShowDevelopEnable =  false
                            viewModel.developEnable()
                        }
                    )
                }
            }
            if (uiState.developEnabled) {
                LinkItem(
                    title = stringResource(id = R.string.settings_developer),
                    icon = R.drawable.settings_developer,
                    listPosition = ListPosition.Last,
                ) {
                    onDevelop()
                }
            }
            Spacer(modifier = Modifier.size(it.calculateBottomPadding()))
        }
    }

    requestPushGrant?.let {
        PushRequest(
            onNotificationEnable = {
            it()
                requestPushGrant = null
            }
        ) { requestPushGrant = null }
    }
}
