package com.gemwallet.android.features.perpetual.views.autoclose

import androidx.compose.animation.AnimatedContentTransitionScope
import androidx.compose.animation.ContentTransform
import androidx.compose.foundation.layout.fillMaxHeight
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
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.confirm.presents.ConfirmScreen
import com.gemwallet.android.features.perpetual.viewmodels.AutocloseViewModel
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.ui.components.animation.navigationSlideTransition
import com.gemwallet.android.ui.models.actions.FinishConfirmAction
import com.gemwallet.android.ui.viewmodel.NavEntryViewModelStoreOwner
import kotlinx.serialization.Serializable

@Composable
fun AutocloseNavGraph(
    onDismiss: () -> Unit,
    finishAction: FinishConfirmAction,
) {
    val rootOwner = rememberAutocloseRootViewModelStoreOwner()
    CompositionLocalProvider(LocalViewModelStoreOwner provides rootOwner) {
        AutocloseNavGraphContent(
            onDismiss = onDismiss,
            finishAction = finishAction,
        )
    }
}

@Composable
private fun AutocloseNavGraphContent(
    onDismiss: () -> Unit,
    finishAction: FinishConfirmAction,
) {
    val viewModel: AutocloseViewModel = hiltViewModel()
    val uiModel by viewModel.uiModel.collectAsStateWithLifecycle()
    val takeProfitText by viewModel.takeProfitText.collectAsStateWithLifecycle()
    val stopLossText by viewModel.stopLossText.collectAsStateWithLifecycle()

    val backStack = remember { mutableStateListOf<NavKey>(AutocloseRoute) }
    var confirmParams by remember { mutableStateOf<ConfirmParams.PerpetualParams?>(null) }

    LaunchedEffect(Unit) {
        viewModel.confirmRequests.collect { params ->
            confirmParams = params
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
            val model = uiModel ?: return@entry
            AutocloseScene(
                model = model,
                takeProfitText = takeProfitText,
                stopLossText = stopLossText,
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
        entry<AutocloseConfirmRoute> {
            confirmParams?.let { params ->
                ConfirmScreen(
                    params = params,
                    cancelAction = popInternal,
                    finishAction = { hash ->
                        finishAction(hash)
                        onDismiss()
                    },
                    onBuy = {},
                    handleSystemBack = true,
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
        modifier = Modifier.fillMaxHeight(0.95f),
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

private class AutocloseNavEntryDecorator(
    private val parent: ViewModelStoreOwner,
    private val savedStateRegistryOwner: SavedStateRegistryOwner,
    private val stores: MutableMap<Any, ViewModelStore>,
) : NavEntryDecorator<NavKey>(
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
