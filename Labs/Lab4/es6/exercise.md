Un programma deve monitorare una macchina leggendo i valori da 10 sensori, che
impiegano tempi diversi per fornire il risultato e sono letti da 10 thread, uno per sensore
(simulare la lettura con una funzione read_value() che fa una sleep di lunghezza casuale e
restituisce un numero casuale compreso tra 0 e 10).
Una volta raccolti i 10 valori, un altro thread raccoglie i risultati e ne esegue la somma. Se il
risultato è maggiore di 50 rallenta la macchina, se inferiore l’accelera (da simulare con una
funzione set_speed() che fa una sleep di lunghezza casuale).
È importante che lettura parametri (ciclo read) e impostazione macchina (ciclo write) non si
sovrappongano, in quanto i valori potrebbero venire perturbati.
Il programma inoltre fa infiniti cicli read/write.
Provare a risolvere il problema utilizzando sia una versione modificata della barriera ciclica
realizzata nell’esercizio 5 che i canali