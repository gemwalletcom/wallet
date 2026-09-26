package com.gemwallet.android.ui

import android.content.Context
import android.content.Intent
import android.content.Intent.FLAG_ACTIVITY_NEW_TASK
import android.os.Build
import androidx.activity.compose.LocalActivity
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
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.core.net.toUri
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.BuildConfig
import com.gemwallet.android.WalletConnectorRequestContent
import com.gemwallet.android.application.wallet_connect.ActiveWalletConnectRequest
import com.gemwallet.android.features.onboarding.presents.OnboardingScene
import com.gemwallet.android.features.onboarding.presents.terms.AcceptTermsDestination
import com.gemwallet.android.flavors.ReviewManager
import com.gemwallet.android.ui.localization.string
import com.gemwallet.android.ui.localization.stringRes
import com.gemwallet.android.ui.navigation.WalletNavGraph
import com.gemwallet.android.ui.navigation.WalletRootRoute
import com.gemwallet.android.ui.navigation.rememberWalletNavigationState
import com.gemwallet.android.ui.navigation.routes.WalletRoute
import uniffi.gemstone.GemAppUpdateAction
import uniffi.gemstone.GemAppUpdateOffer
import uniffi.gemstone.GemNavigationTab

@Composable
fun WalletApp(
    pendingRoutes: List<NavKey> = emptyList(),
    pendingTab: GemNavigationTab? = null,
    onPendingNavigationConsumed: () -> Unit = {},
    onContentReady: () -> Unit = {},
    activeWalletConnectRequest: ActiveWalletConnectRequest? = null,
    onWalletConnectError: (String) -> Unit = {},
    viewModel: AppViewModel = hiltViewModel(),
) {
    val state by viewModel.uiState.collectAsStateWithLifecycle()
    val startDestination by viewModel.startDestinationState.collectAsStateWithLifecycle()
    val isTermsAccepted by viewModel.isTermsAccepted.collectAsStateWithLifecycle()

    val start = startDestination ?: return
    val currentTab = rememberSaveable { mutableStateOf(WalletRoute) }
    val navigator = rememberWalletNavigationState(
        startDestination = start,
        currentTab = currentTab,
        session = viewModel.session,
    )
    val currentOnContentReady by rememberUpdatedState(onContentReady)
    val isWalletRootActive = navigator.backStack.lastOrNull() == WalletRootRoute
    val shouldWaitForWalletRootContent = isWalletRootActive && pendingRoutes.isEmpty()

    LaunchedEffect(pendingRoutes, navigator) {
        if (pendingRoutes.isEmpty()) return@LaunchedEffect
        if (navigator.openPendingNavigation(pendingRoutes, pendingTab)) {
            onPendingNavigationConsumed()
        }
    }

    val walletConnectorRequest = activeWalletConnectRequest?.current?.collectAsStateWithLifecycle()?.value
    LaunchedEffect(walletConnectorRequest?.key, navigator) {
        navigator.showWalletConnectorRequest(walletConnectorRequest?.key)
    }
    val walletConnectorRequestContent: @Composable (String) -> Unit = remember(activeWalletConnectRequest, navigator, onWalletConnectError) {
        { key ->
            activeWalletConnectRequest?.let { activeRequest ->
                WalletConnectorRequestContent(
                    activeRequest = activeRequest,
                    requestKey = key,
                    onGetAsset = navigator::openGetAsset,
                    onOpenAddress = navigator::openAddress,
                    onError = onWalletConnectError,
                )
            }
        }
    }

    WalletNavGraph(
        navigator = navigator,
        onWalletContentReady = onContentReady,
        onAcceptTerms = viewModel::acceptTerms,
        onPayment = viewModel::openPayment,
        walletConnectorRequest = walletConnectorRequestContent,
        onboard = {
            OnboardingScene(
                onCreateWallet = {
                    if (isTermsAccepted) {
                        navigator.openCreateWalletSecurityReminder()
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

    state.update?.let { update ->
        ShowUpdateDialog(
            update = update,
            onSkip = viewModel::onSkip,
            onUpdateOpened = viewModel::onUpdateOpened,
        )
    }

    val activity = LocalActivity.current
    LaunchedEffect(state.intent, activity) {
        if (state.intent == AppIntent.ShowReview && activity != null) {
            viewModel.onReviewOpen()
            ReviewManager().open(activity)
        }
    }
}

@Composable
private fun ShowUpdateDialog(update: GemAppUpdateOffer, onSkip: () -> Unit, onUpdateOpened: () -> Unit) {
    val context = LocalContext.current
    val isPlayStoreInstall = fromGooglePlay(context)
    val isRequired = !update.canSkip()

    if (isPlayStoreInstall && !isRequired) {
        return
    }

    AlertDialog(
        onDismissRequest = {
            if (!isRequired) {
                onSkip()
            }
        },
        confirmButton = {
            TextButton(onClick = {
                openUpdateDestination(context = context, isPlayStoreInstall = isPlayStoreInstall)
                onUpdateOpened()
            }) {
                Text(text = stringResource(id = GemAppUpdateAction.UPDATE.stringRes()))
            }
        },
        dismissButton = if (isRequired) {
            null
        } else {
            {
                TextButton(onClick = onSkip) {
                    Text(text = stringResource(GemAppUpdateAction.SKIP.stringRes()))
                }
            }
        },
        title = {
            Text(text = update.title.string(context))
        },
        text = {
            Text(text = update.description.string(context))
        },
    )
}

private fun openUpdateDestination(context: Context, isPlayStoreInstall: Boolean) {
    val urls = if (isPlayStoreInstall) {
        listOf(
            "market://details?id=${context.packageName}",
            BuildConfig.UPDATE_URL,
        )
    } else {
        listOf(BuildConfig.UPDATE_URL)
    }

    for (uri in urls) {
        val launched = runCatching {
            context.startActivity(
                Intent(Intent.ACTION_VIEW, uri.toUri()).apply {
                    addFlags(FLAG_ACTIVITY_NEW_TASK)
                },
            )
        }.isSuccess
        if (launched) return
    }
}

@Suppress("DEPRECATION")
private fun fromGooglePlay(context: Context): Boolean {
    // A list with valid installers package name
    val validInstallers = listOf("com.android.vending", "com.google.android.feedback")

    val installer = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
        context.packageManager.getInstallSourceInfo(context.packageName).installingPackageName
    } else {
        context.packageManager.getInstallerPackageName(context.packageName)
    }
    return installer != null && validInstallers.contains(installer)
}
