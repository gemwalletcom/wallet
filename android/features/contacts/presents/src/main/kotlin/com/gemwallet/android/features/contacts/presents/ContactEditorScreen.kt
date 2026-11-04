package com.gemwallet.android.features.contacts.presents

import androidx.activity.compose.BackHandler
import androidx.compose.animation.AnimatedContent
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.contacts.viewmodels.ContactEditorViewModel
import com.gemwallet.android.features.contacts.viewmodels.models.ContactEditorPage
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.screen.rememberSnackbarState

@Composable
fun ContactEditorScreen(onSaved: () -> Unit, onCancel: () -> Unit, viewModel: ContactEditorViewModel = hiltViewModel()) {
    val uiState by viewModel.uiState.collectAsStateWithLifecycle()
    val snackbar = rememberSnackbarState(message = uiState.errorText, iconRes = R.drawable.ic_error, onShown = viewModel::clearError)

    BackHandler(uiState.page != ContactEditorPage.Form) { viewModel.back() }

    LaunchedEffect(uiState.saved) {
        if (uiState.saved) {
            onSaved()
        }
    }

    AnimatedContent(targetState = uiState.page, label = "contact_editor") { page ->
        when (page) {
            ContactEditorPage.Form -> ContactEditorScene(
                state = uiState,
                snackbar = snackbar,
                onAction = { action ->
                    when (action) {
                        is ContactEditorAction.SetName -> viewModel.setName(action.name)
                        is ContactEditorAction.SetDescription -> viewModel.setDescription(action.description)
                        ContactEditorAction.SelectAvatar -> viewModel.selectAvatar()
                        ContactEditorAction.RemoveAvatar -> viewModel.removeAvatar()
                        ContactEditorAction.AddAddress -> viewModel.addAddress()
                        is ContactEditorAction.EditAddress -> viewModel.editAddress(action.address)
                        is ContactEditorAction.DeleteAddress -> viewModel.deleteAddress(action.address)
                        ContactEditorAction.Save -> viewModel.save()
                        ContactEditorAction.Cancel -> onCancel()
                    }
                },
            )

            ContactEditorPage.Address -> uiState.addressInput?.let { input ->
                ContactAddressEditorScene(
                    input = input,
                    onAction = { action ->
                        when (action) {
                            is ContactAddressEditorAction.SetAddress -> viewModel.setAddress(action.address)
                            is ContactAddressEditorAction.SetMemo -> viewModel.setMemo(action.memo)
                            is ContactAddressEditorAction.Scan -> viewModel.scanAddress(action.address)
                            is ContactAddressEditorAction.Paste -> viewModel.pasteAddress(action.address)
                            ContactAddressEditorAction.SelectChain -> viewModel.selectChain()
                            ContactAddressEditorAction.ValidateAddress -> viewModel.validateAddress()
                            ContactAddressEditorAction.Confirm -> viewModel.confirmAddress()
                            ContactAddressEditorAction.Cancel -> viewModel.cancelAddress()
                        }
                    },
                )
            }

            ContactEditorPage.SelectChain -> ContactChainSelectScreen(
                onSelect = viewModel::setChain,
                onCancel = viewModel::cancelSelectChain,
            )

            ContactEditorPage.Avatar -> ContactAvatarScene(
                onSelect = viewModel::setAvatar,
                onCancel = viewModel::cancelAvatar,
            )
        }
    }
}
