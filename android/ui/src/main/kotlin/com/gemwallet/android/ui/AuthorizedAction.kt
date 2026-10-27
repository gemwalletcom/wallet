package com.gemwallet.android.ui

import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.currentCompositeKeyHashCode
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.platform.LocalContext
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewmodel.compose.viewModel
import com.gemwallet.android.model.AuthRequest
import kotlinx.coroutines.flow.MutableStateFlow

@Composable
fun rememberAuthorizedAction(auth: AuthRequest, onAuthorized: () -> Unit): () -> Unit {
    val context = LocalContext.current
    val authorization = viewModel<AuthorizationViewModel>(key = "authorization:${currentCompositeKeyHashCode.toString(36)}")
    val isAuthorized by authorization.isAuthorized.collectAsState()

    LaunchedEffect(isAuthorized) {
        if (isAuthorized) {
            authorization.isAuthorized.value = false
            onAuthorized()
        }
    }

    return remember(context, auth, authorization) { { context.requestAuth(auth) { authorization.isAuthorized.value = true } } }
}

internal class AuthorizationViewModel : ViewModel() {
    val isAuthorized = MutableStateFlow(false)
}
