import React, { useState } from 'react';
import { makeStyles } from '@material-ui/core/styles';
import Paper from '@material-ui/core/Paper';
import Tabs from '@material-ui/core/Tabs';
import Tab from '@material-ui/core/Tab';
import { Groups } from '../Groups/Groups';
import { Transactions } from '../Transactions/Transactions';
import { Settings } from '../Settings/Settings';
import { Panels } from './Panels';

const useStyles = makeStyles({
  root: {
    flexGrow: 1,
  },        
});

interface IState {
    selectedValue: number;
};

export const NavigationBar: React.FC = () => {
    const [selectedValue, setSelectedValue] = useState(0);

    return (
        <Paper>
            <Tabs
                value={selectedValue}
                onChange={(evt, value: number) => setSelectedValue(value)}
                indicatorColor="primary"
                textColor="primary"
                centered
            >
                <Tab label="Groups" />
                <Tab label="Transactions" />
                <Tab label="Settings" />
            </Tabs>
            <Panels index={selectedValue}>
                <Groups />
                <Transactions />
                <Settings />
            </Panels>
        </Paper>
    );
}