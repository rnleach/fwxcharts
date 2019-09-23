#
# This script assumes the following have already been set in the gnuplot environment.
# variables:
#   num_hours
#   now_time
#   start_time
#   end_time
#   main_title
#   output_name
#   output_prefix
#
# heredocs:
#   $data
#

#
# Multiplot of fire weather indexes, multiple model runs.
#
reset
#
# Graph style
#
set output output_prefix."/".output_name

# palette
set palette defined (\
0 '#604860',\
1 '#784860',\
2 '#a86060',\
3 '#c07860',\
4 '#f0a848',\
5 '#f8ca8c',\
6 '#feecae',\
7 '#fff4c2',\
8 '#fff7db',\
9 '#fffcf6')
set cbrange [0:(num_hours)]
#
# Get the number of blocks in $data
#
stats $data using 1 nooutput # sets the STATS_blocks variable used below

#
# Set up x axis data
#
set xdata time
set timefmt "%Y-%m-%d-%H"
#
# Set up the multiplot
#
set multiplot layout 4,1 title main_title font ",14"
#
# Plot the top row, which is the HDW
#
set tmargin screen 0.95
set rmargin screen 0.85
set lmargin screen 0.1
set bmargin screen 0.72
set xtics scale 0
set format x ''
set ytics 100,100,700
set grid
unset colorbox
set arrow from now_time, graph 0 to now_time, graph 1 nohead lc rgb "black"
plot [start_time:end_time][0:700] $data u 1:5:2 w l lc palette t "HDW"
#
# Plot the second row which is the blow up dt
#
set tmargin screen 0.72
set rmargin screen 0.85
set lmargin screen 0.1
set bmargin screen 0.49
set xtics scale 0
set ytics 2,2,18
set grid
set arrow from now_time, graph 0 to now_time, graph 1 nohead lc rgb "black"
plot [start_time:end_time][0:20] $data u 1:3:2 w l lc palette t "Blow Up ΔT"
#
# Set up x-axis
#
set xtics nomirror scale 1
set format x "%m/%d %H"
set xtics rotate by -45 offset 0, screen -0.035
set xlabel "Date and hour [UTC]\n" font ",14" offset 0, screen -0.045
#
# Plot the third row which is the blow up height.
#
set tmargin screen 0.49
set rmargin screen 0.85
set lmargin screen 0.1
set bmargin screen 0.26
set xtics scale 0
set ytics 0,2000,15000
set grid
set arrow from now_time, graph 0 to now_time, graph 1 nohead lc rgb "black"
plot [start_time:end_time][0:15000] $data u 1:4:2 w l lc palette t "Blow Up Height"
#
# Clean up
#
unset multiplot
