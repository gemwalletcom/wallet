// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Style
import SwiftUI

struct LockScene: View {
    let model: LockSceneViewModel

    var body: some View {
        placeholderView
            .overlay(alignment: .bottom) { unlockButton }
            .animation(.smooth, value: model.isLocked)
            .frame(maxWidth: .infinity)
    }
}

// MARK: - UI Components

extension LockScene {
    @ViewBuilder
    private var unlockButton: some View {
        if model.isUnlockButtonVisible {
            Button {
                model.startUnlock()
            } label: {
                HStack {
                    if let image = model.unlockImage {
                        Image(systemName: image)
                    }
                    Text(model.unlockTitle)
                }
            }
            .buttonStyle(.blue())
            .frame(maxWidth: .scene.button.maxWidth)
            .padding()
        }
    }

    @ViewBuilder
    private var placeholderView: some View {
        if model.isPasscodeOff {
            StateHeroView(
                systemImage: SystemImage.lockFill,
                title: model.passcodeOffTitle,
                description: model.passcodeOffDescription,
            )
            .padding(.horizontal, .medium)
            .frame(maxWidth: .infinity, maxHeight: .infinity)
            .background(Colors.grayBackground)
        } else {
            LogoView()
                .background(Colors.white)
        }
    }
}
