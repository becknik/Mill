# Schlachtplan - Plan of Action

0.1 Vorwärtszüge-funktion -> Vec an resulting PlayFields

0.2 Rückwärtszüge "

1. Auflistung aller gewonnen (und verlorenen) Konfigurationen
    - Generierungs-Algorithmus für verlorene und gewonnen Konfigurationen
        -> Eine Farbe hat 2 Stein, die andere 3..n mit n = 9 in Produktion, n<9 für Tests
    - Erstellung von Hash-Sets für win & lost Konfiguraitonen
        -> Andere Sets als bei 2.?
    - Was wir für Win/ Loste Konfigs brauchen:
        - Gegner-Farbe hat 2 (beliebig verteilte) Steine, Fargeb hat mindestens eine Mühle auf dem Feld
    - Idee:
        - Erstelle Tabelle mit 2^(12-2) - 1 (erstmal in unboxed array lut)Kombinationen von weißen Steinen zwischen den schwarzen Steinen: BB, BBW, BWB, WBB, BBWW, ...
        - For mögliche_mühlen_position in (1..3) // Mühle äußerer Ring, mittlerer Ring und über Ringe hinweg
            -> Übrige Mühlen ergeben sich aus den Shifts der Kombinationen
        - Definiere 11 Schleifen: Jede Schleife shiftet einen Index aus den Kombinationen der LUT nach links
        - Entstandene Konfigurationen werden in Normalform gebracht und in ein WON-Set eingefügt, wenn sie noch nicht vorhanden sind aufgrund der Spiegelungen
        - Invertiere die Elemente aus dem WON-Set -> LOST-Set

2. Bottom-Up Tree-Walker Algorithmus zum Füllung der HashSets mit allen möglichen Konfigurationen
    - HashSets won/lose füllen
        1. Methode: mark_lost
            - Wird initial auf allen Elementen der generierten Konfigurationen in LOST aufgerufen
            - Für alle Elemente auf dem Stack
            - Finde alle Rückwärtszüge von
        2. Methode: mark_won
    - Rückwärtszug-Enumeration Algorithmus

3. if-Verzweigung für verschiedene Ausgabefälle

4. Schleife zur iteration durch input_felder.txt