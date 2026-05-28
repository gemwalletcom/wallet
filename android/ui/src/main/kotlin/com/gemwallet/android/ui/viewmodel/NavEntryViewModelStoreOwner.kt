package com.gemwallet.android.ui.viewmodel

import androidx.lifecycle.DEFAULT_ARGS_KEY
import androidx.lifecycle.HasDefaultViewModelProviderFactory
import androidx.lifecycle.SAVED_STATE_REGISTRY_OWNER_KEY
import androidx.lifecycle.SavedStateViewModelFactory
import androidx.lifecycle.VIEW_MODEL_STORE_OWNER_KEY
import androidx.lifecycle.ViewModelProvider
import androidx.lifecycle.ViewModelStore
import androidx.lifecycle.ViewModelStoreOwner
import androidx.lifecycle.enableSavedStateHandles
import androidx.lifecycle.viewmodel.CreationExtras
import androidx.lifecycle.viewmodel.MutableCreationExtras
import androidx.savedstate.SavedState
import androidx.savedstate.SavedStateRegistry
import androidx.savedstate.SavedStateRegistryOwner

class NavEntryViewModelStoreOwner(
    private val parent: ViewModelStoreOwner,
    private val store: ViewModelStore,
    private val savedStateRegistryOwner: SavedStateRegistryOwner,
    private val defaultArgs: SavedState,
) : ViewModelStoreOwner,
    SavedStateRegistryOwner,
    HasDefaultViewModelProviderFactory {

    init {
        enableSavedStateHandles()
    }

    override val viewModelStore: ViewModelStore
        get() = store

    override val savedStateRegistry: SavedStateRegistry
        get() = savedStateRegistryOwner.savedStateRegistry

    override val lifecycle
        get() = savedStateRegistryOwner.lifecycle

    override val defaultViewModelProviderFactory: ViewModelProvider.Factory
        get() = (parent as? HasDefaultViewModelProviderFactory)?.defaultViewModelProviderFactory
            ?: SavedStateViewModelFactory()

    override val defaultViewModelCreationExtras: CreationExtras
        get() = MutableCreationExtras(
            (parent as? HasDefaultViewModelProviderFactory)?.defaultViewModelCreationExtras
                ?: CreationExtras.Empty
        ).apply {
            this[SAVED_STATE_REGISTRY_OWNER_KEY] = this@NavEntryViewModelStoreOwner
            this[VIEW_MODEL_STORE_OWNER_KEY] = this@NavEntryViewModelStoreOwner
            this[DEFAULT_ARGS_KEY] = defaultArgs
        }
}
