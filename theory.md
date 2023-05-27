Nash-Gleichgewicht

Mühle-Konfiguration $P \in \{B,W,E\}^{24} \times \{B,W\}$ 

"Gewinnwort" für Weiß: $v_W (P) \{-1, 0, 1\}$
- $1\: \hat{=}$ weiß gewinnt
- $0 \: \hat{=}$ unentschieden
- $-1 \: \hat{=}$ verliert

$v_W (P) = max\: min\: v(P, \beta, \gamma)$ 
- $max: \beta_i$ Stategien von Weiß
- $min: \gamma_i$ Strategien von Schwarz
- $v(P, \beta, \gamma) \in \{-1,0,1\}$: Starte in $P$ und spiele nach Stategie $\beta$ bzw. $\gamma$

win_value$(P \in \{B,W,E\}^{24} \times \{B,W\}, s \in \{B,W\})$ 
- berechne $v_s (P)$
```
MiniMax-Algorithmus:

if is_lost(P,s) then
	return -1;
else if is_won(P,s) then
	return 1;
else if is_draw(P,s) then          //Endlos-Rekursion
	return 0;
else if s ist am Zug (d.h. P ∈ {B,W,E}^24 x {s})
	for z ∈ mögliche Züge (P)
		m = max(win_value(z(P),m))
	return m
else (s nicht am Zug)
	return min{win_value(z(P)) | z ∈ mögliche Züge(P)}

z(P) = mache Zug z
m = max{win_value(z(P)) | z ∈ mögliche Züge(P)}

negamax-Alg: ohne Züge s
```

Idee für Endlos-Rekursion: bei wiederholter Spiel-Konfiguration -> unentschieden
- Man muss sich Züge speichern

Idee: Rückwärts spielen
```
Tiefensuche: 

W,L = {}
for P ∈ LostPositions
	mark_lost(P)
for P ∈ WonPostitions
	mark_won(P)

mark_lost(P)
	if (P !∈ L)
		L:= L ∪ {P}         //Globale Variable
		for z ∈ rückwärtsZüge(P)
			P' = z(P)
			mark_won(P')

mark_won(P)
	if (P !∈ W)
		W:= W ∪ {P}         //Globale Variable
		for z ∈ rückwärtsZüge(P)
			P' = z(P)
			if z'(P') ∈ W für alle z' ∈ vorwärtsZüge(P')
				mark_lost(P')

W,L: HashSet/Map (FNV HashMap)

```

Breitensuche mit Queue Q (nicht rek.)
Per Level mit Distanz

algiers-Account
