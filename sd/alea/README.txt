Alea microSD contents / microSD の内容

Copy this "alea" folder to the root of a FAT32 microSD card.
この alea フォルダを、FAT32 でフォーマットした microSD のルートにコピーします。

/alea/
  config.json            shake detection tuning (optional) / シェイク検出の調整（任意）
                         threshold_mg 200-4000, start_peaks 1-8,
                         start_window_ms 100-3000, end_quiet_ms 100-3000
                         auto_off_min 0-120 (0 = never, default 3)
  tarot/img/NN.a2b       card images and back.a2b (tools/convert_cards.py)
                         カード画像と裏面（tools/convert_cards.py で生成）
  tarot/cap/NN.a1b       card names / カード名（tools/render_tarot.py で生成）
  tarot/word/NN.a1b      keywords / キーワード（tools/render_tarot.py で生成）
                         The tarot files are generated and not in the repository.
                         タロットのファイルは生成物で、リポジトリには含めない。

See DESIGN.md section 7 for details. / 詳細は DESIGN.md §7 を参照。
