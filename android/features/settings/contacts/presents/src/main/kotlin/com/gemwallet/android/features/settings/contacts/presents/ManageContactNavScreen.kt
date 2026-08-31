package com.gemwallet.android.features.settings.contacts.presents

import androidx.compose.animation.AnimatedContent
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.gemwallet.android.features.settings.contacts.viewmodels.ManageContactViewModel
import com.gemwallet.android.features.settings.contacts.viewmodels.models.ManageContactPage

@Composable
fun ManageContactNavScreen(
    onSaved: () -> Unit,
    onCancel: () -> Unit,
    viewModel: ManageContactViewModel = hiltViewModel(),
) {
    val uiState by viewModel.uiState.collectAsStateWithLifecycle()

    LaunchedEffect(uiState.saved) {
        if (uiState.saved) {
            onSaved()
        }
    }

    AnimatedContent(targetState = uiState.page, label = "manage_contact") { page ->
        when (page) {
            ManageContactPage.Form -> ManageContactScene(
                state = uiState,
                onNameChange = viewModel::setName,
                onDescriptionChange = viewModel::setDescription,
                onAction = { action ->
                    when (action) {
                        ManageContactAction.SelectAvatar -> viewModel.selectAvatar()
                        ManageContactAction.RemoveAvatar -> viewModel.removeAvatar()
                        ManageContactAction.AddAddress -> viewModel.addAddress()
                        is ManageContactAction.EditAddress -> viewModel.editAddress(action.address)
                        is ManageContactAction.DeleteAddress -> viewModel.deleteAddress(action.address)
                        ManageContactAction.Save -> viewModel.save()
                        ManageContactAction.Cancel -> onCancel()
                    }
                },
            )

            ManageContactPage.Address -> uiState.addressInput?.let { input ->
                ManageContactAddressScene(
                    input = input,
                    onAddressChange = viewModel::setAddress,
                    onMemoChange = viewModel::setMemo,
                    onScan = viewModel::scanAddress,
                    onPaste = viewModel::pasteAddress,
                    onAction = { action ->
                        when (action) {
                            ManageContactAddressAction.SelectChain -> viewModel.selectChain()
                            ManageContactAddressAction.Confirm -> viewModel.confirmAddress()
                            ManageContactAddressAction.Cancel -> viewModel.cancelAddress()
                        }
                    },
                )
            }

            ManageContactPage.SelectChain -> ContactChainSelectScene(
                onSelect = viewModel::setChain,
                onCancel = viewModel::cancelSelectChain,
            )

            ManageContactPage.Avatar -> ContactAvatarScene(
                onSelect = viewModel::setAvatar,
                onCancel = viewModel::cancelAvatar,
            )
        }
    }
}
