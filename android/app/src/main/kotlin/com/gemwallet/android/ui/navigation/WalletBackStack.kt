package com.gemwallet.android.ui.navigation

import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSerializable
import androidx.navigation3.runtime.NavBackStack
import androidx.navigation3.runtime.NavKey
import androidx.navigation3.runtime.serialization.NavKeySerializer
import kotlinx.serialization.KSerializer
import kotlinx.serialization.builtins.ListSerializer
import kotlinx.serialization.descriptors.SerialDescriptor
import kotlinx.serialization.encoding.Decoder
import kotlinx.serialization.encoding.Encoder

@Composable
internal fun rememberWalletNavBackStack(startDestination: NavKey): NavBackStack<NavKey> {
    val serializer = remember(startDestination) {
        WalletNavBackStackSerializer(startDestination)
    }
    return rememberSerializable(serializer = serializer) {
        NavBackStack(startDestination)
    }
}

private class WalletNavBackStackSerializer(
    private val startDestination: NavKey,
) : KSerializer<NavBackStack<NavKey>> {

    private val delegate = ListSerializer(NavKeySerializer<NavKey>())

    override val descriptor: SerialDescriptor = delegate.descriptor

    override fun serialize(encoder: Encoder, value: NavBackStack<NavKey>) {
        encoder.encodeSerializableValue(
            serializer = delegate,
            value = value.toList().dropNonRestorableRoutes(startDestination),
        )
    }

    override fun deserialize(decoder: Decoder): NavBackStack<NavKey> {
        val routes = decoder
            .decodeSerializableValue(deserializer = delegate)
            .dropNonRestorableRoutes(startDestination)
        return NavBackStack(*routes.toTypedArray())
    }
}

internal fun List<NavKey>.dropNonRestorableRoutes(startDestination: NavKey): List<NavKey> {
    val restoredRoot = firstOrNull()
    val root = restoredRoot
        ?.takeIf { it == startDestination && !it.isNonRestorableRoute() }
        ?: startDestination
    val restoredRoutes = if (restoredRoot == root) drop(1) else emptyList()
    return listOf(root) + restoredRoutes.filterNot { it.isNonRestorableRoute() }
}

private fun NavKey.isNonRestorableRoute(): Boolean {
    return isPendingNavigationProtectedRoute()
}
