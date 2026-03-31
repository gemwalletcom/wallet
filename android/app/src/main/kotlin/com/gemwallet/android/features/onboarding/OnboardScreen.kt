package com.gemwallet.android.features.onboarding

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.widthIn
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.buttons.MainActionButton
import com.gemwallet.android.ui.theme.SceneSizing
import com.gemwallet.android.ui.theme.WalletTheme
import com.gemwallet.android.ui.theme.largeIconSize
import com.gemwallet.android.ui.theme.paddingDefault
import com.gemwallet.android.ui.theme.space24

private const val BrandmarkCornerPercent = 29

@Composable
fun OnboardScreen(
    onCreateWallet: () -> Unit,
    onImportWallet: () -> Unit,
) {
    Box(
        modifier = Modifier
            .background(MaterialTheme.colorScheme.background)
            .fillMaxSize()
    ) {
        Image(
            painterResource(id = R.drawable.brandmark),
            contentDescription = "welcome_icon",
            modifier = Modifier
                .align(Alignment.Center)
                .size(largeIconSize)
                .clip(RoundedCornerShape(BrandmarkCornerPercent)),
        )

        Column(
            horizontalAlignment = Alignment.CenterHorizontally,
            modifier = Modifier
                .align(Alignment.BottomCenter)
                .navigationBarsPadding()
                .padding(bottom = paddingDefault),
        ) {
            MainActionButton(
                title = stringResource(id = R.string.wallet_create_new_wallet),
                modifier = Modifier
                    .widthIn(max = SceneSizing.buttonMaxWidth)
                    .testTag("create"),
                onClick = onCreateWallet,
            )
            Spacer(modifier = Modifier.size(space24))
            MainActionButton(
                title = stringResource(id = R.string.wallet_import_existing_wallet),
                modifier = Modifier
                    .widthIn(max = SceneSizing.buttonMaxWidth)
                    .testTag("import"),
                onClick = onImportWallet,
            )
        }
    }
}

@Preview
@Composable
fun PreviewWelcomeScreen() {
    WalletTheme {
        OnboardScreen(onCreateWallet = { }) {

        }
    }
}
