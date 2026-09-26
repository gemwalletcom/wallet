package com.gemwallet.android.features.perpetuals.presents.autoclose

import androidx.compose.animation.AnimatedContentTransitionScope
import androidx.compose.animation.ContentTransform
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.material3.SnackbarHostState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.DEFAULT_ARGS_KEY
import androidx.lifecycle.HasDefaultViewModelProviderFactory
import androidx.lifecycle.ViewModelStore
import androidx.lifecycle.ViewModelStoreOwner
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.viewmodel.compose.LocalViewModelStoreOwner
import androidx.navigation3.runtime.NavEntryDecorator
import androidx.navigation3.runtime.NavKey
import androidx.navigation3.runtime.entryProvider
import androidx.navigation3.runtime.rememberDecoratedNavEntries
import androidx.navigation3.runtime.rememberSaveableStateHolderNavEntryDecorator
import androidx.navigation3.scene.Scene
import androidx.navigation3.ui.NavDisplay
import androidx.savedstate.SavedStateRegistryOwner
import androidx.savedstate.savedState
import com.gemwallet.android.domains.confirm.ConfirmTransferInput
import com.gemwallet.android.features.assets.presents.address.AddressDetailsScreen
import com.gemwallet.android.features.perpetuals.viewmodels.AutocloseViewModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.animation.navigationSlideTransition
import com.gemwallet.android.ui.components.screen.showSnackbar
import com.gemwallet.android.ui.models.actions.CancelAction
import com.gemwallet.android.ui.models.actions.FinishConfirmAction
import com.gemwallet.android.ui.theme.SheetSizing
import com.gemwallet.android.ui.viewmodel.NavEntryViewModelStoreOwner
import com.wallet.core.primitives.ChainAddress
import kotlinx.serialization.Serializable

typealias AutocloseConfirmContent = @Composable (input: ConfirmTransferInput, finishAction: FinishConfirmAction, cancelAction: CancelAction, onOpenAddress: (ChainAddress) -> Unit) -> Unit

@Composable
fun AutocloseNavGraph(onDismiss: () -> Unit, finishAction: FinishConfirmAction, confirmContent: AutocloseConfirmContent) {
    val rootOwner = rememberAutocloseRootViewModelStoreOwner()
    CompositionLocalProvider(LocalViewModelStoreOwner provides rootOwner) {
        AutocloseNavGraphContent(
            onDismiss = onDismiss,
            finishAction = finishAction,
            confirmContent = confirmContent,
        )
    }
}

@Composable
private fun AutocloseNavGraphContent(onDismiss: () -> Unit, finishAction: FinishConfirmAction, confirmContent: AutocloseConfirmContent) {
    val viewModel: AutocloseViewModel = hiltViewModel()
    val viewState by viewModel.viewState.collectAsStateWithLifecycle()
    val positionRow by viewModel.positionRow.collectAsStateWithLifecycle()

    val backStack = remember { mutableStateListOf<NavKey>(AutocloseRoute) }
    val snackbar = remember { SnackbarHostState() }
    var transfer by remember { mutableStateOf<ConfirmTransferInput?>(null) }

    LaunchedEffect(Unit) {
        viewModel.errors.collect { message -> snackbar.showSnackbar(message, R.drawable.ic_error) }
    }

    LaunchedEffect(Unit) {
        viewModel.confirmRequests.collect { input ->
            transfer = input
            if (backStack.lastOrNull() != AutocloseConfirmRoute) {
                backStack.add(AutocloseConfirmRoute)
            }
        }
    }

    val popInternal = {
        if (backStack.size > 1) backStack.removeAt(backStack.lastIndex)
    }

    val entryProvider = entryProvider<NavKey> {
        entry<AutocloseRoute> {
            val model = viewState ?: return@entry
            AutocloseScene(
                model = model,
                positionRow = positionRow,
                snackbar = snackbar,
                onAction = { action ->
                    when (action) {
                        AutocloseAction.Close -> onDismiss()
                        AutocloseAction.Confirm -> viewModel.onConfirm()
                        is AutocloseAction.TakeProfitChanged -> viewModel.onTakeProfitChanged(action.text)
                        is AutocloseAction.StopLossChanged -> viewModel.onStopLossChanged(action.text)
                        is AutocloseAction.SelectPercent -> viewModel.onPercentSelected(action.type, action.percent)
                    }
                },
            )
        }
        entry<AutocloseAddressDetailsRoute> { route ->
            AddressDetailsScreen(
                chainAddress = route.chainAddress,
                onCancel = popInternal,
            )
        }
        entry<AutocloseConfirmRoute> {
            transfer?.let { input ->
                confirmContent(
                    input,
                    FinishConfirmAction { hash, warning ->
                        finishAction(hash, warning)
                        onDismiss()
                    },
                    CancelAction { popInternal() },
                    { chainAddress ->
                        val route = AutocloseAddressDetailsRoute(chainAddress)
                        if (backStack.lastOrNull() != route) {
                            backStack.add(route)
                        }
                    },
                )
            }
        }
    }

    val decoratedEntries = rememberDecoratedNavEntries(
        entries = backStack.map { entryProvider(it) },
        entryDecorators = listOf(
            rememberSaveableStateHolderNavEntryDecorator(),
            rememberAutocloseNavEntryDecorator(),
        ),
    )

    NavDisplay(
        entries = decoratedEntries,
        modifier = Modifier.fillMaxHeight(SheetSizing.heightFraction),
        onBack = {
            if (backStack.size > 1) popInternal() else onDismiss()
        },
        transitionSpec = slideLeftTransition,
        popTransitionSpec = slideRightTransition,
        predictivePopTransitionSpec = { slideRightTransition() },
    )
}

@Serializable
private data object AutocloseRoute : NavKey

@Serializable
private data object AutocloseConfirmRoute : NavKey

@Serializable
private data class AutocloseAddressDetailsRoute(val chainAddress: ChainAddress) : NavKey

private typealias AutocloseNavTransition =
    AnimatedContentTransitionScope<Scene<NavKey>>.() -> ContentTransform

private val slideLeftTransition: AutocloseNavTransition = {
    navigationSlideTransition(AnimatedContentTransitionScope.SlideDirection.Left)
}

private val slideRightTransition: AutocloseNavTransition = {
    navigationSlideTransition(AnimatedContentTransitionScope.SlideDirection.Right)
}

@Composable
private fun rememberAutocloseRootViewModelStoreOwner(): ViewModelStoreOwner {
    val parentOwner = checkNotNull(LocalViewModelStoreOwner.current) {
        "No ViewModelStoreOwner via LocalViewModelStoreOwner"
    }
    val savedStateRegistryOwner = checkNotNull(parentOwner as? SavedStateRegistryOwner) {
        "Parent ViewModelStoreOwner must implement SavedStateRegistryOwner"
    }
    val parentArgs = (parentOwner as? HasDefaultViewModelProviderFactory)
        ?.defaultViewModelCreationExtras
        ?.get(DEFAULT_ARGS_KEY)
        ?: savedState()
    val store = remember { ViewModelStore() }
    DisposableEffect(store) {
        onDispose { store.clear() }
    }
    return remember(parentOwner, store, savedStateRegistryOwner, parentArgs) {
        NavEntryViewModelStoreOwner(
            parent = parentOwner,
            store = store,
            savedStateRegistryOwner = savedStateRegistryOwner,
            defaultArgs = parentArgs,
        )
    }
}

@Composable
private fun rememberAutocloseNavEntryDecorator(): NavEntryDecorator<NavKey> {
    val parentOwner = checkNotNull(LocalViewModelStoreOwner.current) {
        "No ViewModelStoreOwner via LocalViewModelStoreOwner"
    }
    val savedStateRegistryOwner = checkNotNull(parentOwner as? SavedStateRegistryOwner) {
        "Parent ViewModelStoreOwner must implement SavedStateRegistryOwner"
    }
    val stores = remember { mutableMapOf<Any, ViewModelStore>() }
    DisposableEffect(Unit) {
        onDispose {
            stores.values.forEach(ViewModelStore::clear)
            stores.clear()
        }
    }
    return remember(parentOwner, savedStateRegistryOwner) {
        AutocloseNavEntryDecorator(parentOwner, savedStateRegistryOwner, stores)
    }
}

private class AutocloseNavEntryDecorator(private val parent: ViewModelStoreOwner, private val savedStateRegistryOwner: SavedStateRegistryOwner, private val stores: MutableMap<Any, ViewModelStore>) :
    NavEntryDecorator<NavKey>(
        onPop = { contentKey -> stores.remove(contentKey)?.clear() },
        decorate = { entry ->
            val store = remember(entry.contentKey) {
                stores.getOrPut(entry.contentKey) { ViewModelStore() }
            }
            val owner = remember(parent, store, savedStateRegistryOwner) {
                NavEntryViewModelStoreOwner(
                    parent = parent,
                    store = store,
                    savedStateRegistryOwner = savedStateRegistryOwner,
                    defaultArgs = savedState(),
                )
            }
            CompositionLocalProvider(LocalViewModelStoreOwner provides owner) {
                entry.Content()
            }
        },
    )
