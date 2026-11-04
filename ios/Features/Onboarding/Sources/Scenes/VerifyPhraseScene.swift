// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Localization
import PrimitivesComponents
import Style
import SwiftUI

struct VerifyPhraseScene: View {
    @State private var model: VerifyPhraseSceneViewModel

    init(model: VerifyPhraseSceneViewModel) {
        _model = State(initialValue: model)
    }

    var body: some View {
        List {
            CalloutView(style: .header(title: Localized.SecretPhrase.Confirm.QuickTest.title))
                .cleanListRow()

            Section {
                CaptureProtectedView {
                    SecretPhraseGridView(
                        words: model.verifiedWords,
                        highlightIndex: model.wordsIndex,
                    )
                }
            }
            .cleanListRow()

            Section {
                CaptureProtectedView {
                    Grid(alignment: .center) {
                        ForEach(model.groups, id: \.self) { section in
                            GridRow(alignment: .center) {
                                ForEach(section) { row in
                                    WordChipView(word: row.word, isVerified: model.isVerified(index: row)) {
                                        model.pickWord(index: row)
                                    }
                                }
                            }
                        }
                    }
                }
            }
            .cleanListRow()
        }
        .safeAreaButton {
            StateButton(
                text: Localized.Common.continue,
                state: model.buttonState,
                action: model.onContinue,
            )
        }
        .contentMargins([.top], .extraSmall, for: .scrollContent)
        .listSectionSpacing(.custom(.medium))
        .navigationTitle(model.title)
        .alertSheet($model.isPresentingAlertMessage)
        .detectScreenshots(kind: .phrase)
        .protectFromScreenRecording()
    }
}
