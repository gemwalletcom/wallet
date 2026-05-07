package com.gemwallet.android.ui

import android.content.Context
import android.content.Intent
import android.content.Intent.FLAG_ACTIVITY_NEW_TASK
import android.os.Build
import androidx.activity.compose.LocalActivity
import androidx.compose.foundation.layout.Row
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.core.net.toUri
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.BuildConfig
import com.gemwallet.android.ext.updateUrl
import com.gemwallet.android.features.onboarding.OnboardScreen
import com.gemwallet.android.flavors.ReviewManager
import com.gemwallet.android.ui.components.PushRequest
import com.gemwallet.android.features.onboarding.AcceptTermsDestination
import com.gemwallet.android.ui.navigation.WalletNavGraph
import com.gemwallet.android.ui.navigation.WalletRootRoute
import com.gemwallet.android.ui.navigation.rememberWalletNavigationState
import com.gemwallet.android.ui.models.actions.AssetIdAction
import com.gemwallet.android.ui.navigation.routes.assetsRoute
import com.gemwallet.android.ui.theme.Spacer16

@Composable
fun WalletApp(
    pendingRoutes: List<NavKey> = emptyList(),
    onIntentConsumed: () -> Unit = {},
    onContentReady: () -> Unit = {},
    walletConnectOverlay: @Composable (AssetIdAction) -> Unit = {},
    viewModel: AppViewModel = hiltViewModel(),
) {
    val state by viewModel.uiState.collectAsStateWithLifecycle()
    val startDestination by viewModel.startDestinationState.collectAsStateWithLifecycle()
    val askNotifications by viewModel.askNotifications.collectAsStateWithLifecycle()
    val isTermsAccepted by viewModel.isTermsAccepted.collectAsStateWithLifecycle()

    val start = startDestination ?: return
    val currentTab = rememberSaveable { mutableStateOf(assetsRoute) }
    val navigator = rememberWalletNavigationState(startDestination = start, currentTab = currentTab)
    val onBuy = remember(navigator) { AssetIdAction { navigator.openBuy(it) } }
    var confirmPendingNavigation by remember(pendingRoutes) { mutableStateOf(false) }
    val currentOnContentReady by rememberUpdatedState(onContentReady)
    val isWalletRootActive = navigator.backStack.lastOrNull() == WalletRootRoute
    val shouldWaitForWalletRootContent = isWalletRootActive && pendingRoutes.isEmpty()

    LaunchedEffect(pendingRoutes, navigator, confirmPendingNavigation) {
        if (pendingRoutes.isEmpty()) return@LaunchedEffect
        if (confirmPendingNavigation) {
            return@LaunchedEffect
        }
        if (navigator.openPendingNavigation(pendingRoutes)) {
            onIntentConsumed()
        } else if (navigator.needsPendingNavigationConfirmation()) {
            confirmPendingNavigation = true
        }
    }

    WalletNavGraph(
        navigator = navigator,
        onWalletContentReady = onContentReady,
        onAcceptTerms = viewModel::acceptTerms,
        onboard = {
            OnboardScreen(
                onCreateWallet = {
                    if (isTermsAccepted) {
                        navigator.openCreateWalletRules()
                    } else {
                        navigator.openAcceptTerms(AcceptTermsDestination.Create)
                    }
                },
                onImportWallet = {
                    if (isTermsAccepted) {
                        navigator.openImportWallet()
                    } else {
                        navigator.openAcceptTerms(AcceptTermsDestination.Import)
                    }
                },
            )
        },
    )

    LaunchedEffect(shouldWaitForWalletRootContent) {
        if (!shouldWaitForWalletRootContent) {
            currentOnContentReady()
        }
    }

    walletConnectOverlay(onBuy)
    state.update?.let { update ->
        ShowUpdateDialog(
            version = update.version,
            isRequired = update.isRequired,
            onSkip = viewModel::onSkip,
            onCancel = viewModel::onCancelUpdate
        )
    }

    val activity = LocalActivity.current
    LaunchedEffect(state.intent, activity) {
        if (state.intent == AppIntent.ShowReview && activity != null) {
            viewModel.onReviewOpen()
            ReviewManager().open(activity)
        }
    }

    if (askNotifications) {
        PushRequest(
            onNotificationEnable = viewModel::onNotificationsEnable,
            onDismiss = viewModel::laterAskNotifications,
        )
    }

    if (confirmPendingNavigation && pendingRoutes.isNotEmpty()) {
        OpenPendingNavigationDialog(
            onOpen = {
                confirmPendingNavigation = false
                if (navigator.openPendingNavigation(pendingRoutes, confirmed = true)) {
                    onIntentConsumed()
                }
            },
            onCancel = {
                confirmPendingNavigation = false
                onIntentConsumed()
            },
        )
    }
}

@Composable
private fun OpenPendingNavigationDialog(
    onOpen: () -> Unit,
    onCancel: () -> Unit,
) {
    AlertDialog(
        onDismissRequest = onCancel,
        confirmButton = {
            TextButton(onClick = onOpen) {
                Text(text = stringResource(id = R.string.common_continue))
            }
        },
        dismissButton = {
            TextButton(onClick = onCancel) {
                Text(text = stringResource(id = R.string.common_cancel))
            }
        },
        title = {
            Text(text = stringResource(id = R.string.common_warning))
        },
    )
}

@Composable
private fun ShowUpdateDialog(
    version: String,
    isRequired: Boolean,
    onSkip: () -> Unit,
    onCancel: () -> Unit,
) {
    val context = LocalContext.current
    val isPlayStoreInstall = fromGooglePlay(context)

    if (isPlayStoreInstall && !isRequired) {
        return
    }

    AlertDialog(
        onDismissRequest = {
            if (!isRequired) {
                onCancel()
            }
        },
        confirmButton = {
            TextButton(onClick = {
                openUpdateDestination(
                    context = context,
                    version = version,
                    isPlayStoreInstall = isPlayStoreInstall,
                )
                if (!isRequired) {
                    onCancel()
                }
            }) {
                Text(text = stringResource(id = R.string.update_app_action))
            }
        },
        dismissButton = if (isRequired) {
            null
        } else {
            {
                Row {
                    TextButton(onClick = onCancel) {
                        Text(text = stringResource(id = R.string.common_cancel))
                    }
                    Spacer16()
                    TextButton(onClick = onSkip) {
                        Text(text = stringResource(R.string.common_skip))
                    }
                }
            }
        },
        title = {
            Text(text = stringResource(id = R.string.update_app_title))
        },
        text = {
            Text(text = stringResource(id = R.string.update_app_description, version))
        }
    )
}

private fun openUpdateDestination(
    context: Context,
    version: String,
    isPlayStoreInstall: Boolean,
) {
    val urls = if (isPlayStoreInstall) {
        listOf(
            "market://details?id=${context.packageName}",
            BuildConfig.UPDATE_URL,
        )
    } else {
        listOf(
            updateUrl(
                flavor = BuildConfig.FLAVOR,
                version = version,
                fallbackUrl = BuildConfig.UPDATE_URL,
            )
        )
    }

    for (uri in urls) {
        val launched = runCatching {
            context.startActivity(
                Intent(Intent.ACTION_VIEW, uri.toUri()).apply {
                    addFlags(FLAG_ACTIVITY_NEW_TASK)
                }
            )
        }.isSuccess
        if (launched) return
    }
}

@Suppress("DEPRECATION")
private fun fromGooglePlay(context: Context): Boolean {
    // A list with valid installers package name
    val validInstallers = listOf("com.android.vending", "com.google.android.feedback")

    val installer = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R){
        context.packageManager.getInstallSourceInfo(context.packageName).installingPackageName
    } else{
        context.packageManager.getInstallerPackageName(context.packageName)
    }
    return installer != null && validInstallers.contains(installer)
}
