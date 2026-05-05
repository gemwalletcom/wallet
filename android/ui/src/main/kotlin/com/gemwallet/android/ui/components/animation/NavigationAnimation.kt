package com.gemwallet.android.ui.components.animation

import androidx.compose.animation.AnimatedContentTransitionScope
import androidx.compose.animation.ContentTransform
import androidx.compose.animation.EnterTransition
import androidx.compose.animation.ExitTransition
import androidx.compose.animation.core.tween
import androidx.compose.animation.togetherWith

private const val NavigationDurationMillis = 300

object NavigationAnimation {
    fun tabContentTransition(): ContentTransform =
        EnterTransition.None togetherWith ExitTransition.None
}

fun <T> AnimatedContentTransitionScope<T>.navigationSlideTransition(
    forward: Boolean,
): ContentTransform =
    navigationSlideTransition(
        direction = if (forward) {
            AnimatedContentTransitionScope.SlideDirection.Left
        } else {
            AnimatedContentTransitionScope.SlideDirection.Right
        }
    )

fun <T> AnimatedContentTransitionScope<T>.navigationSlideTransition(
    direction: AnimatedContentTransitionScope.SlideDirection,
): ContentTransform =
    slideIntoContainer(
        towards = direction,
        animationSpec = tween(NavigationDurationMillis),
    ) togetherWith slideOutOfContainer(
        towards = direction,
        animationSpec = tween(NavigationDurationMillis),
    )
