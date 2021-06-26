# Plot with e.g. gnuplot -p -e "filename='compube-1.csv'" histo.gnuplot

set terminal wxt size 1500,800

binwidth=5
bin(x,width)=width*floor(x/width)

# set logscale y

plot filename using (bin($1,binwidth)):(1.0) smooth freq with boxes
