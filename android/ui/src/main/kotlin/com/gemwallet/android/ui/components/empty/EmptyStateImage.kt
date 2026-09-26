package com.gemwallet.android.ui.components.empty

import androidx.annotation.DrawableRes

sealed interface EmptyStateImage {
    @JvmInline value class Drawable(@DrawableRes val id: Int) : EmptyStateImage

    @JvmInline value class Vector(@DrawableRes val id: Int) : EmptyStateImage
}
