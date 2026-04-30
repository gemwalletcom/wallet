// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Style
import SwiftUI

struct LockScreenScene: View {
    @State private var model: LockSceneViewModel

    init(model: LockSceneViewModel) {
        self.model = model
    }

    var body: some View {
        ZStack {
            placeholderView
                .overlay(alignment: .bottom) {
                    unlockButton
                }
                .onAppear {
                    if model.isLocked {
                        unlock()
                    }
                }
        }
        .animation(.smooth, value: model.isLocked)
        .frame(maxWidth: .infinity)
        .onChange(of: model.isLocked) { _, newState in
            if newState {
                unlock()
            }
        }
    }
}

// MARK: - UI Components

extension LockScreenScene {
    private var unlockButton: some View {
        VStack(spacing: .medium) {
            if model.state == .lockedCanceled {
                Button(action: unlock) {
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
    }

    var placeholderView: some View {
        LogoView()
            .background(Colors.white)
    }
}

// MARK: - Actions

extension LockScreenScene {
    private func unlock() {
        Task {
            await model.unlock()
        }
    }
}

// MARK: - Previews

#Preview {
    LockScreenScene(model: .init())
}
