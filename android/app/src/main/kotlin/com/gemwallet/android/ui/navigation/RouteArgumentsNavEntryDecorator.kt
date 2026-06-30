package com.gemwallet.android.ui.navigation

import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.remember
import androidx.lifecycle.ViewModel
import androidx.lifecycle.ViewModelProvider
import androidx.lifecycle.ViewModelStore
import androidx.lifecycle.ViewModelStoreOwner
import androidx.lifecycle.viewmodel.CreationExtras
import androidx.lifecycle.viewmodel.compose.LocalViewModelStoreOwner
import androidx.navigation3.runtime.NavEntry
import androidx.navigation3.runtime.NavEntryDecorator
import androidx.navigation3.runtime.NavKey
import androidx.navigation3.runtime.NavMetadataKey
import androidx.navigation3.runtime.get
import androidx.navigation3.runtime.metadata
import androidx.savedstate.SavedState
import androidx.savedstate.SavedStateRegistryOwner
import androidx.savedstate.compose.LocalSavedStateRegistryOwner
import androidx.savedstate.savedState
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.gemwallet.android.ui.viewmodel.NavEntryViewModelStoreOwner
import com.wallet.core.primitives.AssetId
import kotlin.reflect.KClass

private object RouteArgumentsKey : NavMetadataKey<Map<String, Any>> {
    override fun toString(): String = "routeArguments"
}

internal fun routeArguments(vararg arguments: Pair<RouteArgument, Any?>): Map<String, Any> {
    val values = arguments
        .mapNotNull { (argument, value) -> value?.let { argument.key to it } }
        .toMap()

    if (values.isEmpty()) {
        return emptyMap()
    }

    return metadata {
        put(RouteArgumentsKey, values)
    }
}

internal fun assetIdArgument(assetId: AssetId): Pair<RouteArgument, String> =
    RouteArgument.AssetId to assetId.toIdentifier()

internal fun fiatAmountArgument(amount: Double?): Pair<RouteArgument, Double?> =
    RouteArgument.FiatAmount to amount

internal fun contactIdArgument(contactId: String): Pair<RouteArgument, String> =
    RouteArgument.ContactId to contactId

internal fun fromAssetIdArgument(assetId: AssetId?): Pair<RouteArgument, String?> =
    RouteArgument.FromAssetId to assetId?.toIdentifier()

internal fun toAssetIdArgument(assetId: AssetId?): Pair<RouteArgument, String?> =
    RouteArgument.ToAssetId to assetId?.toIdentifier()

internal fun paramsArgument(params: String): Pair<RouteArgument, String> =
    RouteArgument.Params to params

@Composable
internal fun <T : Any> rememberRouteArgumentsViewModelStoreNavEntryDecorator(
    viewModelStoreOwner: ViewModelStoreOwner = checkNotNull(LocalViewModelStoreOwner.current) {
        "No ViewModelStoreOwner was provided via LocalViewModelStoreOwner"
    }
): NavEntryDecorator<T> {
    val entryViewModelStores = rememberEntryViewModelStores(viewModelStoreOwner)
    return remember(viewModelStoreOwner, entryViewModelStores) {
        RouteArgumentsViewModelStoreNavEntryDecorator(
            parent = viewModelStoreOwner,
            entryViewModelStores = entryViewModelStores,
        )
    }
}

private class RouteArgumentsViewModelStoreNavEntryDecorator<T : Any>(
    private val parent: ViewModelStoreOwner,
    private val entryViewModelStores: EntryViewModelStores,
) : NavEntryDecorator<T>(
    onPop = { contentKey -> entryViewModelStores.clear(contentKey) },
    decorate = { entry ->
        val routeArguments = entry.metadata[RouteArgumentsKey].orEmpty()
        val defaultArgs = remember(routeArguments) { savedState(routeArguments) }
        val owner = rememberEntryViewModelStoreOwner(
            contentKey = entry.contentKey,
            parent = parent,
            entryViewModelStores = entryViewModelStores,
            defaultArgs = defaultArgs,
        )

        CompositionLocalProvider(LocalViewModelStoreOwner provides owner) {
            entry.Content()
        }
    },
)

@Composable
private fun rememberEntryViewModelStoreOwner(
    contentKey: Any,
    parent: ViewModelStoreOwner,
    entryViewModelStores: EntryViewModelStores,
    defaultArgs: SavedState,
    savedStateRegistryOwner: SavedStateRegistryOwner? = LocalSavedStateRegistryOwner.current,
): ViewModelStoreOwner {
    val store = remember(entryViewModelStores, contentKey) {
        entryViewModelStores.store(contentKey)
    }
    return remember(parent, store, defaultArgs, savedStateRegistryOwner) {
        NavEntryViewModelStoreOwner(
            parent = parent,
            store = store,
            savedStateRegistryOwner = checkNotNull(savedStateRegistryOwner) {
                "No SavedStateRegistryOwner was provided via LocalSavedStateRegistryOwner"
            },
            defaultArgs = defaultArgs,
        )
    }
}

@Composable
private fun rememberEntryViewModelStores(parent: ViewModelStoreOwner): EntryViewModelStores {
    return remember(parent) {
        ViewModelProvider.create(
            store = parent.viewModelStore,
            factory = EntryViewModelStoresFactory,
        )[EntryViewModelStores::class]
    }
}

private class EntryViewModelStores : ViewModel() {
    private val stores = mutableMapOf<Any, ViewModelStore>()

    fun store(contentKey: Any): ViewModelStore {
        return stores.getOrPut(contentKey) { ViewModelStore() }
    }

    fun clear(contentKey: Any) {
        stores.remove(contentKey)?.clear()
    }

    override fun onCleared() {
        stores.values.forEach(ViewModelStore::clear)
        stores.clear()
    }
}

private object EntryViewModelStoresFactory : ViewModelProvider.Factory {
    override fun <T : ViewModel> create(modelClass: KClass<T>, extras: CreationExtras): T {
        require(modelClass == EntryViewModelStores::class) {
            "Unsupported ViewModel: $modelClass"
        }
        @Suppress("UNCHECKED_CAST")
        return EntryViewModelStores() as T
    }
}

internal fun NavEntry<NavKey>.withOccurrenceContentKey(
    key: NavKey,
    occurrence: Int,
): NavEntry<NavKey> {
    val uniqueContentKey = if (occurrence == 0) contentKey else "$contentKey#$occurrence"
    val entry = this
    return NavEntry(
        key = key,
        contentKey = uniqueContentKey,
        metadata = metadata,
    ) {
        entry.Content()
    }
}
